package com.github.yoep.video.vlc;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Platform;

/**
 * Minimal interface to the standard "C" library.
 */
public interface LibC extends Library {

    /**
     * Native library instance.
     */
    LibC INSTANCE = Native.load((Platform.isWindows() ? "msvcrt" : "c"), LibC.class);

    /**
     * Change or add an evironment variable.
     * <p>
     * The value strings are copied (natively).
     * <p>
     * <em>Not available on Windows.</em>
     *
     * @param name      name of environment variable
     * @param value     value of the environment variable
     * @param overwrite non-zero to replace any existing value
     * @return 0 if successful; -1 if not, setting <code>errno</code> to an error code
     */
    int setenv(String name, String value, int overwrite);

    /**
     * Closest Windows equivalent to {@link #setenv(String, String, int)}.
     * <p>
     * Note that after setting an environment variable, it will <em>not</em> show up via
     * System#getenv even if it was successfully set.
     * <p>
     * Use with case, it is not guaranteed to be thread-safe.
     * <p>
     * <em>Only available on Windows.</em>
     *
     * @param envstring variable and value to set, in the format "variable=value", without quotes.
     * @return zero on success, non-zero on error
     */
    int _putenv(String envstring);
}
