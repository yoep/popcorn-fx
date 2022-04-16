package com.github.yoep.player.chromecast.discovery;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.File;
import java.io.IOException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

class FfmpegDiscoveryTest {
    @TempDir
    File workingDirectory;

    @Test
    void testDiscoverProbe_whenPathPropertyHasBeenSet_shouldReturnProbeFromPathWhenFound() throws IOException {
        System.setProperty(FfmpegDiscovery.FFMPEG_PATH_PROPERTY, workingDirectory.getAbsolutePath());
        var expectedFile = new File(workingDirectory.getAbsolutePath() + File.separator + FfmpegDiscovery.PROBE_FILENAME);
        expectedFile.createNewFile();

        FfmpegDiscovery.discoverProbe();
        var result = FfmpegDiscovery.PROBE_DISCOVERED_AT.get();

        assertNotNull(result, "Expected the probe to have been discovered");
        assertEquals(FfmpegDiscovery.DiscoveredAt.PROPERTY_PATH, result);
    }

    @Test
    void testDiscoverProbe_whenWorkingDirectoryContainsProbe_shouldReturnProbeFromWorkingDirectory() throws IOException {
        System.setProperty(FfmpegDiscovery.FFMPEG_WORKING_DIR_PROPERTY, workingDirectory.getAbsolutePath());
        var expectedFile = new File(workingDirectory.getAbsolutePath() + File.separator + FfmpegDiscovery.PROBE_FILENAME);
        expectedFile.createNewFile();

        FfmpegDiscovery.discoverProbe();
        var result = FfmpegDiscovery.PROBE_DISCOVERED_AT.get();

        assertNotNull(result, "Expected the probe to have been discovered");
        assertEquals(FfmpegDiscovery.DiscoveredAt.WORKING_DIRECTORY, result);
    }

    @Test
    void testDiscoverProbe_whenProbeWasNotDiscoveredAtPathOrWorkingDirectory_shouldReturnSystemPathProbe() {
        FfmpegDiscovery.discoverProbe();
        var result = FfmpegDiscovery.PROBE_DISCOVERED_AT.get();

        assertNotNull(result, "Expected the probe to have been discovered");
        assertEquals(FfmpegDiscovery.DiscoveredAt.SYSTEM_PATH, result);
    }
}