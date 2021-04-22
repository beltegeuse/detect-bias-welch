# Detecting Bias in Monte Carlo Renderers using Welch's t-test
by Alisa Jung, Johannes Hanika, Carsten Dachsbacher

Paper: http://jcgt.org/published/0009/02/01/


Translation in Rust of original author's code from me with some minor modification on the command line. 

This tool is to check if two rendered images will converge or not to the same image. This technique is usually more reliable than visualizing the error difference and does not need that one image is entirely converged (a.k.a, reference image). Please read the paper to see in which cases the output of this algorithm can be trusted.

To use this code, you need to save from your renderer the sum and sum of squared values of Welch samples by averaging samples over an image block (please look at the paper for more details).

Here the command line client accepts PFM images directly. Imagine you have:
 - `out1_welch1.pfm`: First algorithm output which contains the sum of welch samples.
 - `out1_welch2.pfm`: First algorithm output which contains the sum of welch samples squared. 
 - `out2_welch1.pfm`: First algorithm output which contains the sum of welch samples.
 - `out2_welch2.pfm`: First algorithm output which contains the sum of welch samples squared.

Then you can call:

```cargo run --release --example=cli -- out1_welch1.pfm out1_welch2.pfm 10 out2_welch1.pfm out2_welch2.pfm 10```

where the 10 refer to how many samples is used to produce these images from the first and second aglrithm. Optionally the output can be scale using `-s`.

## TODO
- Missing output histogram of p-values
- Missing option to visualize each p-values for R,G,B