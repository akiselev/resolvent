module examples.nonlinear_heat;

model NonlinearHeat {
    domain Omega {
        dimension = 2;
        coordinates = cartesian;
    }

    field T: state scalar H1(order=1) on Omega {
        quantity = ThermodynamicTemperature;
        unit = K;
        nominal = 300 K;
        time_role = differential;
    };

    property rho = density(T);
    property cp = specific_heat(T);
    property k = thermal_conductivity(T);
    source Q: VolumetricHeatSource;

    equation energy on Omega {
        rho * cp * dt(T) - div(k * grad(T)) = Q;
    }

    initial {
        T = exact_T(0);
    }

    boundary walls on boundary("walls") {
        dirichlet T = exact_T(t);
    }

    observable total_energy {
        integrate(rho * cp * T);
    }
}
