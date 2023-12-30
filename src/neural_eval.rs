use ndarray::prelude::*;

const SCALE: i16 = 1000;

#[derive(Debug)]
struct NeuralNetwork {
    w_dense0: Array<i16, Ix1>,
    b_dense0: Array<i16, Ix1>,
    w_dense1: Array<i16, Ix1>,
    b_dense1: Array<i16, Ix1>,
    w_dense2: Array<i16, Ix1>,
    b_dense2: Array<i16, Ix1>,
    w_output: Array<i16, Ix1>,
    b_output: Array<i16, Ix1>,
}

impl NeuralNetwork {
    pub fn new() -> Self {
        NeuralNetwork {
            w_dense0: Array::zeros(512*768),
            b_dense0: Array::zeros(512),
            w_dense1: Array::zeros(256*512),
            b_dense1: Array::zeros(256),
            w_dense2: Array::zeros(32*265),
            b_dense2: Array::zeros(32),
            w_output: Array::zeros(1*32),
            b_output: Array::zeros(1),
        }
    }

    pub fn load_weights(
        &mut self,
        w0: Array<i16, Ix1>,
        b0: Array<i16, Ix1>,
        w1: Array<i16, Ix1>,
        b1: Array<i16, Ix1>,
        w2: Array<i16, Ix1>,
        b2: Array<i16, Ix1>,
        w3: Array<i16, Ix1>,
        b3: Array<i16, Ix1>,
    ) {
        self.w_dense0 = w0;
        self.b_dense0 = b0;
        self.w_dense1 = w1;
        self.b_dense1 = b1;
        self.w_dense2 = w2;
        self.b_dense2 = b2;
        self.w_output = w3;
        self.b_output = b3;
    }

    pub fn pred(&self, input: &Array<i16, Ix1>) -> i16 {
        // Input layer
        let mut output = input.clone();

        // Dense layer 0
        output = self.w_dense0.dot(&output) + &self.b_dense0;
        output.mapv_inplace(relu);

        // Dense layer 1
        output = self.w_dense1.dot(&output) + &self.b_dense1;
        output.mapv_inplace(relu);

        // Dense layer 2
        output = self.w_dense2.dot(&output) + &self.b_dense2;
        output.mapv_inplace(relu);

        // Output layer
        output = self.w_output.dot(&output) + &self.b_output;

        output[0]
    }
}

fn relu(x: i16) -> i16 {
    x.max(0)
}
