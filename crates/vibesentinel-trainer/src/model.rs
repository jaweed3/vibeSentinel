use burn::{
    module::Module,
    nn::{Linear, LinearConfig, Relu, Sigmoid},
    tensor::{backend::Backend, Tensor},
};

#[derive(Module, Debug)]
pub struct VibeSentinelAutoencoder<B: Backend> {
    pub enc1: Linear<B>,
    pub enc2: Linear<B>,
    pub dec1: Linear<B>,
    pub dec2: Linear<B>,
    relu: Relu,
    sigmoid: Sigmoid,
}

impl<B: Backend> VibeSentinelAutoencoder<B> {
    pub fn new(device: &B::Device) -> Self {
        Self {
            enc1:    LinearConfig::new(20, 10).init(device),
            enc2:    LinearConfig::new(10, 4).init(device),
            dec1:    LinearConfig::new(4, 10).init(device),
            dec2:    LinearConfig::new(10, 20).init(device),
            relu:    Relu::new(),
            sigmoid: Sigmoid::new(),
        }
    }

    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let h1 = self.relu.forward(self.enc1.forward(x));
        let z  = self.relu.forward(self.enc2.forward(h1));
        let h2 = self.relu.forward(self.dec1.forward(z));
        self.sigmoid.forward(self.dec2.forward(h2))
    }

    pub fn loss(&self, x: Tensor<B, 2>) -> Tensor<B, 1> {
        let recon = self.forward(x.clone());
        let diff = x - recon;
        (diff.clone() * diff).mean_dim(1).squeeze::<1>(1)
    }
}
