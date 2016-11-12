//! Constructions related to feed-forward networks

use std::cmp::min;

use num::{Float, zero};

use {Compute, BackpropTrain, SupervisedTrain};
use activations::ActivationFunction;
use training::{PerceptronRule, GradientDescent};

/// A feedforward layer
///
/// Such layer is composed of a set of output neurons, and have all its
/// inputs connected to all its outputs.
///
/// The effective computation is thus, if `X` is the vector of inputs,
/// `Y` the vector of outputs, `W` the internal weigths matrix, `B` the vector
/// of biases and `f()` the activation function (applied on all components of
/// the vector in parallel):
///
/// ```text
/// Y = f( W*X + B )
/// ```
///
/// The training of this layer consists on fitting the values of `W` and `B`.
pub struct FeedforwardLayer<F: Float, V: Fn(F) -> F, D: Fn(F) -> F> {
    inputs: usize,
    coeffs: Vec<F>,
    biases: Vec<F>,
    activation: ActivationFunction<F, V, D>
}

impl<F, V, D> FeedforwardLayer<F, V, D>
    where F: Float,
          V: Fn(F) -> F,
          D: Fn(F) -> F
{
    /// Creates a new linear feedforward layer with all its weights set
    /// to 0 and its biases set to 0
    pub fn new(inputs: usize,
               outputs: usize,
               activation: ActivationFunction<F, V, D>)
        -> FeedforwardLayer<F, V, D>
    {
        FeedforwardLayer {
            inputs: inputs,
            coeffs: vec![zero(); inputs*outputs],
            biases: vec![zero(); outputs],
            activation: activation
        }
    }

    /// Creates a new linear feedforward layer with all its weights and biases
    /// generated by provided closure (for example a random number generator).
    pub fn new_from<G>(inputs: usize,
                       outputs: usize,
                       activation: ActivationFunction<F, V, D>,
                       mut generator: G)
        -> FeedforwardLayer<F, V, D>
        where G: FnMut() -> F
    {
        FeedforwardLayer {
            inputs: inputs,
            coeffs: (0..inputs*outputs).map(|_| generator()).collect(),
            biases: (0..outputs).map(|_| generator()).collect(),
            activation: activation
        }
    }

    /// Creates a new linear feedforward layer with all its weights and biases
    /// generated by provided closures (one for weights, one for biases).
    pub fn new_from_generators<G>(inputs: usize,
                                  outputs: usize,
                                  activation: ActivationFunction<F, V, D>,
                                  mut weight_generator: G,
                                  mut bias_generator: G)
        -> FeedforwardLayer<F, V, D>
        where G: FnMut() -> F
    {
        FeedforwardLayer {
            inputs: inputs,
            coeffs: (0..inputs*outputs).map(|_| weight_generator()).collect(),
            biases: (0..outputs).map(|_| bias_generator()).collect(),
            activation: activation
        }
    }

    /// Creates a new linear feedforward layer with its weights and biases provided
    pub fn new_from_values(inputs: usize,
                           outputs: usize,
                           activation: ActivationFunction<F, V, D>,
                           coefficients: Vec<F>,
                           biases: Vec<F>)
        -> FeedforwardLayer<F, V, D>
    {
        FeedforwardLayer {
            inputs: inputs,
            coeffs: coefficients,
            biases: biases,
            activation: activation
        }
    }

    pub fn get_coefficients(&self) -> &Vec<F>
    {
        &self.coeffs
    }

    pub fn set_coefficients(&mut self, coefficients: Vec<F>)
    {
        //TODO: Some validation should be done or
        // some values should be re-initialized
        self.coeffs = coefficients;
    }

    pub fn get_biases(&self) -> &Vec<F>
    {
        &self.biases
    }

    pub fn set_biases(&mut self, biases: Vec<F>)
    {

        //TODO: Some validation should be done or
        // some values should be re-initialized
        self.biases = biases;
    }
 
}

impl<F, V, D> Compute<F> for FeedforwardLayer<F, V, D>
    where F: Float,
          V: Fn(F) -> F,
          D: Fn(F) -> F
{
    fn compute(&self, input: &[F]) -> Vec<F> {
        let mut out = self.biases.clone();
        for j in 0..self.biases.len() {
            for i in 0..min(self.inputs, input.len()) {
                out[j] = out[j] + self.coeffs[j*self.inputs + i] * input[i]
            }
        }
        
        for o in &mut out {
            *o = (self.activation.value)(*o);
        }

        out
    }

    fn input_size(&self) -> usize {
        self.inputs
    }

    fn output_size(&self) -> usize {
        self.biases.len()
    }
}

impl<F, V, D> SupervisedTrain<F, PerceptronRule<F>> for FeedforwardLayer<F, V, D>
    where F: Float,
          V: Fn(F) -> F,
          D: Fn(F) -> F
{
    fn supervised_train(&mut self,
                        rule: &PerceptronRule<F>,
                        input: &[F],
                        target: &[F])
    {
        let out = self.compute(input);
        for j in 0..self.biases.len() {
            let diff = out[j] - target.get(j).map(|v| *v).unwrap_or(zero());
            for i in 0..min(self.inputs, input.len()) {
                self.coeffs[i + j*self.inputs] =
                    self.coeffs[i + j*self.inputs] - rule.rate * diff * input[i];
            }
            self.biases[j] = self.biases[j] - rule.rate * diff;
        }
    }
}

impl<F, V, D> BackpropTrain<F, GradientDescent<F>> for FeedforwardLayer<F, V, D>
    where F: Float,
          V: Fn(F) -> F,
          D: Fn(F) -> F
{
    fn backprop_train(&mut self,
                      rule: &GradientDescent<F>,
                      input: &[F],
                      target: &[F])
        -> Vec<F>
    {
        // we need to compute the intermediate states
        let mut out = self.biases.clone();
        for j in 0..self.biases.len() {
            for i in 0..min(self.inputs, input.len()) {
                out[j] = out[j] + self.coeffs[j*self.inputs + i] * input[i]
            }
        }

        let deltas = out.iter()
                            .map(|x| { (self.activation.derivative)(*x) })
                            .collect::<Vec<_>>();
        for o in &mut out {
            *o = (self.activation.value)(*o);
        }

        let mut returned = input.to_owned();
        for j in 0..self.biases.len() {
            for i in 0..min(self.inputs, input.len()) {
                returned[i] = returned[i] - self.coeffs[i + j*self.inputs]*deltas[j];
                self.coeffs[i + j*self.inputs] =
                    self.coeffs[i + j*self.inputs]
                    - rule.rate * input.get(i).map(|x| *x).unwrap_or(zero())
                                * deltas[j]
                                * ( out[j] - target.get(j).map(|x| *x).unwrap_or(zero()) )

            }
            self.biases[j] = self.biases[j]
                    - rule.rate * deltas[j]
                                * ( out[j] - target.get(j).map(|x| *x).unwrap_or(zero()) );
        }
        returned
    }
}

impl<F, V, D> SupervisedTrain<F, GradientDescent<F>> for FeedforwardLayer<F, V, D>
    where F: Float,
          V: Fn(F) -> F,
          D: Fn(F) -> F
{
    fn supervised_train(&mut self,
                        rule: &GradientDescent<F>,
                        input: &[F],
                        target: &[F])
    {
        self.backprop_train(rule, input, target);
    }
}

#[cfg(test)]
mod tests {

    use {Compute, SupervisedTrain};
    use activations::{identity, step, sigmoid};
    use training::{PerceptronRule, GradientDescent};
    use util::Chain;

    use super::FeedforwardLayer;

    #[test]
    fn basics() {
        let layer = FeedforwardLayer::<f32, _, _>::new(7, 3, identity());
        assert_eq!(layer.input_size(), 7);
        assert_eq!(layer.output_size(), 3);
    }

    #[test]
    fn compute() {
        let layer = FeedforwardLayer::new_from(4, 2, identity(), || 0.5f32);
        let output = layer.compute(&[1.0, 1.0, 1.0, 1.0]);
        // all weigths and biases are 0.5, output should be 4*0.5 + 0.5 = 2.5
        for o in &output {
            assert!((o - 2.5).abs() < 0.00001);
        }
    }

    #[test]
    fn perceptron_rule() {
        let mut layer = FeedforwardLayer::new(4, 2, step());
        let rule = PerceptronRule { rate: 0.5f32 };
        for _ in 0..3 {
            layer.supervised_train(&rule, &[1.0,1.0,1.0,1.0], &[0.0, 0.0]);
            layer.supervised_train(&rule, &[1.0,-1.0,1.0,-1.0], &[1.0, 1.0]);
        }
        assert_eq!(layer.compute(&[1.0, 1.0, 1.0, 1.0]), [0.0f32, 0.0]);
        assert_eq!(layer.compute(&[1.0, -1.0, 1.0, -1.0]), [1.0f32, 1.0]);
    }

    #[test]
    fn supervised_train() {
        // a deterministic pseudo-random initialization.
        // uniform init is actually terrible for neural networks.
        let random = {
            let mut acc = 0;
            move || { acc += 1; (1.0f32 + ((13*acc) % 12) as f32) / 13.0f32}
        };
        let mut layer = FeedforwardLayer::new_from(4, 2, sigmoid(), random);
        let rule = GradientDescent { rate: 0.5f32 };
        for _ in 0..40 {
            layer.supervised_train(&rule, &[1.0,1.0,1.0,1.0], &[0.0, 0.0]);
            layer.supervised_train(&rule, &[1.0,-1.0,1.0,-1.0], &[1.0, 1.0]);
        }
        assert!({ let out = layer.compute(&[1.0, 1.0, 1.0, 1.0]); out[0] < 0.2 && out[1] < 0.2 });
        assert!({ let out = layer.compute(&[1.0, -1.0, 1.0, -1.0]); out[0] > 0.8 && out[1] > 0.8 });
    }

    #[test]
    fn backprop_train() {
        // a deterministic pseudo-random initialization.
        // uniform init is actually terrible for neural networks.
        let mut random = {
            let mut acc = 0;
            move || { acc += 1; (1.0f32 + ((13*acc) % 12) as f32) / 13.0f32}
        };
        let mut layer = Chain::new(FeedforwardLayer::new_from(4, 8, sigmoid(), &mut random), FeedforwardLayer::new_from(8, 2, sigmoid(), &mut random));
        let rule = GradientDescent { rate: 0.5f32 };
        for _ in 0..200 {
            layer.supervised_train(&rule, &[1.0, 1.0,1.0, 1.0], &[1.0, 0.0]);
            layer.supervised_train(&rule, &[1.0,-1.0,1.0,-1.0], &[0.0, 1.0]);
        }
        println!("{:?}", layer.compute(&[1.0, 1.0, 1.0, 1.0]));
        assert!({ let out = layer.compute(&[1.0, 1.0, 1.0, 1.0]); out[0] > 0.8 && out[1] < 0.2 });
        println!("{:?}", layer.compute(&[1.0, -1.0, 1.0, -1.0]));
        assert!({ let out = layer.compute(&[1.0, -1.0, 1.0, -1.0]); out[0] < 0.2 && out[1] > 0.8 });
    }
}
