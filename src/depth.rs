struct DepthBuffer<P, D> {
    vec: Vec<(P, D)>,
    width: usize
}

impl<P: Default + Clone, D: Default + PartialOrd + Clone> DepthBuffer<P, D> {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            vec: vec![(P::default(), D::default()); width * height],
        }
    }

    fn depth_at(&self, x: usize, y: usize) -> &D {
        &self.vec[x + y * self.width].1
    }

    fn add(&mut self, x: usize, y: usize, pixel: P, depth: D) {
        let index = x + y * self.width;

        if depth > self.vec[index].1 {
            self.vec[index] = (pixel, depth)
        }
    }

    fn iter(&self) -> ::std::slice::Iter<(P, D)> {
        self.vec.iter()
    }
}