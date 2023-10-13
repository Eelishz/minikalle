use shakmaty::{Bitboard, Board, Chess, Color, Outcome, Position, Role};
use tensorflow::{Graph, SavedModelBundle, SessionOptions, SessionRunArgs, Tensor};

const POSITIVE_INFINITY: i32 = 999999999;
const NEGATIVE_INFINITY: i32 = -999999999;

pub struct Evaluator {
    bundle: SavedModelBundle,
    graph: Graph,
}

impl Evaluator {
    pub fn new() -> Self {
        // Initialize save_dir, input tensor, and an empty graph
        let save_dir = "model";

        let mut graph = Graph::new();

        // Load saved model bundle (session state + meta_graph data)
        let bundle =
            SavedModelBundle::load(&SessionOptions::new(), &["serve"], &mut graph, save_dir)
                .expect("Can't load saved model");

        Evaluator { bundle, graph }
    }

    fn encode_board(&self, board: &Board) -> Vec<f32> {
        fn bitboard_to_vec(bitboard: Bitboard) -> Vec<f32> {
            let mut vec = Vec::from([0.0 as f32; 64]);
            for (i, square) in bitboard.into_iter().enumerate() {
                vec[i] = (square as i32 as f32).clamp(0.0, 1.0);
            }
            vec
        }
        let mut vec: Vec<f32> = vec![];
        for color in Color::ALL {
            for role in Role::ALL {
                vec.append(&mut bitboard_to_vec(
                    board.by_color(color).intersect(board.by_role(role)),
                ))
            }
        }
        vec
    }

    pub fn score(&self, position: Chess) -> i32 {
        let tensor: Tensor<f32> = Tensor::new(&[1, 12, 8, 8])
            .with_values(&self.encode_board(position.board()))
            .expect("Can't create tensor");

        (self.run_model(&self.bundle, &self.graph, &tensor) * 1_000.0) as i32
    }

    pub fn run_model(&self, bundle: &SavedModelBundle, graph: &Graph, tensor: &Tensor<f32>) -> f32 {
        // Get the session from the loaded model bundle
        let session = &bundle.session;

        let signature_input_parameter_name = "data_in";
        let signature_output_parameter_name = "data_out";

        // Get signature metadata from the model bundle
        let signature = bundle
            .meta_graph_def()
            .get_signature("serving_default")
            .unwrap();

        // Get input/output info
        let input_info = signature.get_input(signature_input_parameter_name).unwrap();
        let output_info = signature
            .get_output(signature_output_parameter_name)
            .unwrap();

        // Get input/output ops from graph
        let input_op = graph
            .operation_by_name_required(&input_info.name().name)
            .unwrap();
        let output_op = graph
            .operation_by_name_required(&output_info.name().name)
            .unwrap();

        // Manages inputs and outputs for the execution of the graph
        let mut args = SessionRunArgs::new();
        args.add_feed(&input_op, 0, &tensor); // Add any inputs

        let out = args.request_fetch(&output_op, 0); // Request outputs

        // Run model
        session
            .run(&mut args) // Pass to session to run
            .expect("Error occurred during calculations");

        // Fetch outputs after graph execution
        let out_res: f32 = args.fetch(out).unwrap()[0];

        out_res
    }

    fn count_pieces(&self, position: Chess) -> i32 {
        let white_material = position.board().material_side(Color::White);
        let black_material = position.board().material_side(Color::Black);

        white_material.pawn as i32 * 100
            + white_material.knight as i32 * 350
            + white_material.bishop as i32 * 300
            + white_material.rook as i32 * 500
            + white_material.queen as i32 * 900
            - black_material.pawn as i32 * 100
            - black_material.knight as i32 * 350
            - black_material.bishop as i32 * 300
            - black_material.rook as i32 * 500
            - black_material.queen as i32 * 900
    }

    pub fn evaluate(&self, position: Chess, depth_from_root: u8) -> i32 {
        let evaluation = match position.outcome() {
            Some(Outcome::Draw) => return 0,
            Some(Outcome::Decisive { winner }) => {
                if winner == Color::White {
                    POSITIVE_INFINITY - depth_from_root as i32
                } else {
                    NEGATIVE_INFINITY + depth_from_root as i32
                }
            }
            // None => self.score(position.clone()),
            None => self.count_pieces(position.clone()), /* + self.score(position.clone())*/
        };
        if position.turn() == Color::Black {
            return  -evaluation;
        }
        return  evaluation;
    }
}
