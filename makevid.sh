#!/bin/sh

ffmpeg -pattern_type sequence -start_number 0 -r 1 -i out-%03d.png -s 600x600 -c:v libx264 -vf fps=25 -pix_fmt yuv420p test.mp4
