#!/bin/bash

rm ../src/model/*
python export_weights.py
mv *.in ../src/model/
