#pragma once
#include <vector>
#include <stdexcept>

struct RiskCoord {
    double r;    // normalized [0,1]
    double w;    // weight >= 0
};

struct RiskState {
    std::vector<RiskCoord> coords;
    double V; // residual

    static RiskState from_raw(const std::vector<RiskCoord>& rc) {
        if (rc.empty()) throw std::runtime_error("No risk coordinates (no corridor -> no deployment)");
        RiskState s;
        s.coords = rc;
        s.V = 0.0;
        for (const auto& c : s.coords) {
            if (c.r < 0.0 || c.r > 1.0) throw std::runtime_error("r out of [0,1]");
            if (c.w < 0.0) throw std::runtime_error("negative weight");
            s.V += c.r * c.w;
        }
        return s;
    }

    RiskState next(const std::vector<RiskCoord>& rc_next) const {
        RiskState n = from_raw(rc_next);
        if (n.V > V + 1e-9) {
            throw std::runtime_error("Lyapunov residual increased (auto-derate/stop)");
        }
        return n;
    }
};
