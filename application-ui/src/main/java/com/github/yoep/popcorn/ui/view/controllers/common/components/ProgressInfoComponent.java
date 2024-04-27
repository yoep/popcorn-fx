package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.ui.torrent.utils.SizeUtils;
import com.github.yoep.popcorn.ui.utils.ProgressUtils;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import lombok.Getter;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;

@Slf4j
@Getter
public class ProgressInfoComponent {
    @FXML
    Label progressPercentage;
    @FXML
    Label downloadText;
    @FXML
    Label uploadText;
    @FXML
    Label activePeersText;
    @FXML
    Label downloadedText;

    /**
     * Updates the UI components based on the provided download status.
     *
     * @param status the download status to update the UI with
     */
    public void update(DownloadStatus status) {
        Objects.requireNonNull(status, "status cannot be null");
        progressPercentage.setText(ProgressUtils.progressToPercentage(status));
        downloadText.setText(ProgressUtils.progressToDownload(status));
        uploadText.setText(ProgressUtils.progressToUpload(status));
        activePeersText.setText(String.valueOf(status.seeds()));
        downloadedText.setText(SizeUtils.toDisplaySize(status.downloaded()));
    }
}
