package dev.nucleus.spring;

import dev.nucleus.NucleusClient;
import dev.nucleus.NucleusClaims;
import jakarta.servlet.*;
import jakarta.servlet.http.*;
import java.io.IOException;

public class NucleusAuthFilter implements Filter {
    private final NucleusClient client;

    public NucleusAuthFilter(NucleusClient client) { this.client = client; }

    @Override
    public void doFilter(ServletRequest req, ServletResponse res, FilterChain chain) throws IOException, ServletException {
        HttpServletRequest httpReq = (HttpServletRequest) req;
        String auth = httpReq.getHeader("Authorization");
        if (auth != null && auth.startsWith("Bearer ")) {
            try {
                NucleusClaims claims = client.verifyToken(auth.substring(7));
                httpReq.setAttribute("nucleusClaims", claims);
            } catch (Exception ignored) {}
        }
        chain.doFilter(req, res);
    }
}
