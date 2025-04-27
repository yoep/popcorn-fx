package com.github.yoep.popcorn.ui.utils;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Loading;
import com.github.yoep.popcorn.ui.torrent.utils.SizeUtils;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Objects;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class ProgressUtils {
    public static String progressToPercentage(DownloadStatus status) {
        Objects.requireNonNull(status, "status cannot be null");
        return String.format("%1$,.2f", status.progress() * 100) + "%";
    }

    public static String progressToDownload(DownloadStatus status) {
        Objects.requireNonNull(status, "status cannot be null");
        return SizeUtils.toDisplaySize(status.downloadSpeed()) + "/s";
    }

    public static String progressToUpload(DownloadStatus status) {
        Objects.requireNonNull(status, "status cannot be null");
        return SizeUtils.toDisplaySize(status.uploadSpeed()) + "/s";
    }

    public static String progressToPercentage(Loading.Progress status) {
        Objects.requireNonNull(status, "status cannot be null");
        return String.format("%1$,.2f", status.getProgress() * 100) + "%";
    }

    public static String progressToDownload(Loading.Progress status) {
        Objects.requireNonNull(status, "status cannot be null");
        return SizeUtils.toDisplaySize(status.getDownloadSpeed()) + "/s";
    }

    public static String progressToUpload(Loading.Progress status) {
        Objects.requireNonNull(status, "status cannot be null");
        return SizeUtils.toDisplaySize(status.getUploadSpeed()) + "/s";
    }
}
