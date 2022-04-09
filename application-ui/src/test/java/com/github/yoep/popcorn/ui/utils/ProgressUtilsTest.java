package com.github.yoep.popcorn.ui.utils;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class ProgressUtilsTest {
    @Test
    void testProgressToPercentage_whenStatusIsGiven_shouldReturnTheExpectedResult() {
        var status = new SimpleDownloadStatus() {
            @Override
            public float getProgress() {
                return 0.205f;
            }
        };
        var expectedResult = "20.50%";

        var result = ProgressUtils.progressToPercentage(status);

        assertEquals(expectedResult, result);
    }

    @Test
    void testProgressToDownload_whenStatusIsGiven_shouldReturnExpectedResult() {
        var status = new SimpleDownloadStatus() {
            @Override
            public int getDownloadSpeed() {
                return 1024;
            }
        };
        var expectedResult = "1.00 KB/s";

        var result = ProgressUtils.progressToDownload(status);

        assertEquals(expectedResult, result);
    }

    @Test
    void testProgressToUpload_whenStatusIsGiven_shouldReturnExpectedResult() {
        var status = new SimpleDownloadStatus() {
            @Override
            public int getUploadSpeed() {
                return 2048;
            }
        };
        var expectedResult = "2.00 KB/s";

        var result = ProgressUtils.progressToUpload(status);

        assertEquals(expectedResult, result);
    }

    static class SimpleDownloadStatus implements DownloadStatus {
        @Override
        public float getProgress() {
            return 0;
        }

        @Override
        public int getSeeds() {
            return 0;
        }

        @Override
        public int getDownloadSpeed() {
            return 0;
        }

        @Override
        public int getUploadSpeed() {
            return 0;
        }

        @Override
        public long getDownloaded() {
            return 0;
        }

        @Override
        public long getTotalSize() {
            return 0;
        }
    }
}