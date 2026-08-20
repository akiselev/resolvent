module acceptance.macro;
model HeatLike {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field T: state scalar H1(order=1) on Omega { time_role = differential; };
  property k = 2 + 0.01 * T;
  equation energy on Omega { dt(T) - div(k * grad(T)) = 0; }
  boundary wall on boundary("wall") { dirichlet T = 300; }
  observable mean_T { integrate(T); }
}
