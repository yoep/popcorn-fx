package com.github.yoep.popcorn.ui.utils;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.loader.LoadingProgress;
import org.junit.jupiter.api.Test;

import java.text.DecimalFormatSymbols;
import java.text.MessageFormat;

import static org.junit.jupiter.api.Assertions.assertEquals;

class ProgressUtilsTest {
    @Test
    void testProgressToPercentage_whenStatusIsGiven_shouldReturnTheExpectedResult() {
        var status = new SimpleDownloadStatus() {
            @Override
            public float progress() {
                return 0.205f;
            }
        };
        var expectedResult = MessageFormat.format("20{0}50%", DecimalFormatSymbols.getInstance().getDecimalSeparator());

        var result = ProgressUtils.progressToPercentage(status);

        assertEquals(expectedResult, result);
    }

    @Test
    void testProgressToDownload_whenStatusIsGiven_shouldReturnExpectedResult() {
        var status = new SimpleDownloadStatus() {
            @Override
            public int downloadSpeed() {
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
            public int uploadSpeed() {
                return 2048;
            }
        };
        var expectedResult = "2.00 KB/s";

        var result = ProgressUtils.progressToUpload(status);

        assertEquals(expectedResult, result);
    }

    @Test
    void testLoadingProgressToPercentage_whenStatusIsGiven_shouldReturnTheExpectedResult() {
        var status = new LoadingProgress();
        status.progress = 0.205f;
        var expectedResult = MessageFormat.format("20{0}50%", DecimalFormatSymbols.getInstance().getDecimalSeparator());

        var result = ProgressUtils.progressToPercentage(status);

        assertEquals(expectedResult, result);
    }

    @Test
    void testLoadingProgressToDownload_whenStatusIsGiven_shouldReturnExpectedResult() {
        var status = new LoadingProgress();
        status.downloadSpeed = 1024;
        var expectedResult = "1.00 KB/s";

        var result = ProgressUtils.progressToDownload(status);

        assertEquals(expectedResult, result);
    }

    @Test
    void testLoadingProgressToUpload_whenStatusIsGiven_shouldReturnExpectedResult() {
        var status = new LoadingProgress();
        status.uploadSpeed = 2048;
        var expectedResult = "2.00 KB/s";

        var result = ProgressUtils.progressToUpload(status);

        assertEquals(expectedResult, result);
    }

    static class SimpleDownloadStatus implements DownloadStatus {
        @Override
        public float progress() {
            return 0;
        }

        @Override
        public int seeds() {
            return 0;
        }

        @Override
        public int peers() {
            return 0;
        }

        @Override
        public int downloadSpeed() {
            return 0;
        }

        @Override
        public int uploadSpeed() {
            return 0;
        }

        @Override
        public long downloaded() {
            return 0;
        }

        @Override
        public long totalSize() {
            return 0;
        }
    }
}