package com.github.yoep.popcorn.backend.adapters.platform;

/**
 * The platform info describes the current platform information.
 */
public interface PlatformInfo {
    /**
     * Get the current platform system information type.
     *
     * @return Returns the OS type.
     */
    PlatformType getType();

    /**
     * Get the current platform system architecture type.
     *
     * @return Returns the OS architecture.
     */
    String getArch();
}
