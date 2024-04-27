package com.github.yoep.popcorn.backend.utils;

import com.github.yoep.popcorn.backend.PopcornException;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.io.IOException;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.UnknownHostException;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class HostUtils {
    /**
     * Retrieve the host address of the current machine.
     *
     * @return Returns the host address.
     * @throws PopcornException Is thrown when the host address couldn't be retrieved.
     */
    
    public static String hostAddress() {
        try {
            return InetAddress.getLocalHost().getHostAddress();
        } catch (UnknownHostException e) {
            throw new PopcornException(e.getMessage(), e);
        }
    }

    /**
     * Retrieve an available port on the current machine.
     * This port will always be above 1000 as the ports below require administrative privileges.
     *
     * @return Returns the available port.
     */
    public static int availablePort() {
        var port = 2000;
        var available = false;

        while (!available) {
            port++;

            try (var ignored = new ServerSocket(port)) {
                available = true;
            } catch (IOException ex) {
                // no-op
            }
        }

        return port;
    }
}
