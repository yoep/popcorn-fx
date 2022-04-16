package com.github.yoep.player.chromecast.discovery;

import com.github.kokorin.jaffree.ffprobe.FFprobe;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.util.Arrays;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class FfmpegDiscovery {
    static final String FFMPEG_PATH_PROPERTY = "ffmpeg.path";
    static final String FFMPEG_WORKING_DIR_PROPERTY = "user.dir";
    static final String PROBE_FILENAME = "ffprobe";

    static AtomicReference<DiscoveredAt> PROBE_DISCOVERED_AT = new AtomicReference<>();

    public static FFprobe discoverProbe() {
        var ffprobe = discoverFromPath()
                .orElseGet(() -> discoverFromWorkingDirectory()
                        .orElseGet(FfmpegDiscovery::useSystemPathDiscovery));

        log.info("Loaded ffprobe from {}", PROBE_DISCOVERED_AT.get());
        return ffprobe;
    }

    private static Optional<FFprobe> discoverFromPath() {
        return Optional.ofNullable(System.getProperty(FFMPEG_PATH_PROPERTY))
                .map(e -> {
                    log.debug("FFMPEG path property has been configured at {}, checking for {}", e, PROBE_FILENAME);
                    return e;
                })
                .map(File::new)
                .filter(FfmpegDiscovery::directoryContainsProbe)
                .map(e -> {
                    log.debug("{} has been found in {}", PROBE_FILENAME, e.getAbsolutePath());
                    PROBE_DISCOVERED_AT.set(DiscoveredAt.PROPERTY_PATH);
                    return e;
                })
                .map(File::toPath)
                .map(FFprobe::atPath);
    }

    private static Optional<FFprobe> discoverFromWorkingDirectory() {
        return Optional.ofNullable(System.getProperty(FFMPEG_WORKING_DIR_PROPERTY))
                .map(e -> {
                    log.debug("Checking for {} within the working directory {}", PROBE_FILENAME, e);
                    return e;
                })
                .map(File::new)
                .filter(FfmpegDiscovery::directoryContainsProbe)
                .map(e -> {
                    log.debug("{} has been found in {}", PROBE_FILENAME, e.getAbsolutePath());
                    PROBE_DISCOVERED_AT.set(DiscoveredAt.WORKING_DIRECTORY);
                    return e;
                })
                .map(File::toPath)
                .map(FFprobe::atPath);
    }

    private static FFprobe useSystemPathDiscovery() {
        log.debug("Using default system path discovery for {}", PROBE_FILENAME);
        PROBE_DISCOVERED_AT.set(DiscoveredAt.SYSTEM_PATH);
        return FFprobe.atPath();
    }

    private static boolean directoryContainsProbe(File directory) {
        return Optional.of(directory)
                .filter(File::exists)
                .map(File::listFiles)
                .flatMap(FfmpegDiscovery::containsProbeFile)
                .orElse(false);
    }

    private static Optional<Boolean> containsProbeFile(File[] files) {
        return Arrays.stream(files)
                .filter(File::isFile)
                .filter(file -> file.getName().startsWith(PROBE_FILENAME))
                .findFirst()
                .map(file -> true);
    }

    enum DiscoveredAt {
        PROPERTY_PATH,
        WORKING_DIRECTORY,
        SYSTEM_PATH
    }

}
