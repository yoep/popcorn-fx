package com.github.yoep.player.popcorn.controls;

import com.github.yoep.player.popcorn.utils.SizeUtils;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import javafx.application.Platform;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.geometry.Bounds;
import javafx.geometry.HPos;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.control.PopupControl;
import javafx.scene.control.Skin;
import javafx.scene.layout.ColumnConstraints;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import javafx.scene.layout.RowConstraints;

import java.text.MessageFormat;
import java.util.function.Function;

public class StreamInfo extends Icon {
    public static final String FACTORY_PROPERTY = "factory";
    public static final String DOWNLOAD_CELL = "download";
    public static final String UPLOAD_CELL = "upload";
    public static final String ACTIVE_PEERS_CELL = "active_peers";
    public static final String DOWNLOADED_CELL = "downloaded";

    private static final String STYLE_CLASS = "stream-info";
    private static final String POPUP_STYLE_CLASS = "info-popup";
    private static final String VALUE_STYLE_CLASS = "value";

    private final ObjectProperty<Function<String, StreamInfoCell>> factory = new SimpleObjectProperty<>(this, FACTORY_PROPERTY, StreamInfoCell::new);
    private final StreamPopup popup = new StreamPopup();

    private boolean firstRender = true;

    //region Constructors

    /**
     * Initialize a new instance of stream info.
     */
    public StreamInfo() {
        super(Icon.EYE_UNICODE);
        init();
    }

    /**
     * Initialize a new instance of stream info.
     *
     * @param unicode The icon unicode to use.
     */
    public StreamInfo(String unicode) {
        super(unicode);
        init();
    }

    //endregion

    //region Properties

    public Function<String, StreamInfoCell> getFactory() {
        return factory.get();
    }

    public ObjectProperty<Function<String, StreamInfoCell>> factoryProperty() {
        return factory;
    }

    public void setFactory(Function<String, StreamInfoCell> factory) {
        this.factory.set(factory);
    }

    //endregion

    //region Methods

    /**
     * Check if the stream info is being showed.
     *
     * @return Returns true if the stream info popup is shown, else false.
     */
    public boolean isShowing() {
        return popup.isShowing();
    }

    /**
     * Update the stream info with the given stream progress information.
     *
     * @param progress The update information to display.
     */
    public void update(DownloadStatus progress) {
        popup.updateInfo(progress);
    }

    /**
     * Show the stream info popup.
     */
    public void show() {
        Bounds screenBounds = this.localToScreen(this.getBoundsInLocal());
        double x = screenBounds.getMaxX();
        double y = screenBounds.getMinY();

        // show below the icon
        y += getHeight();
        // show centered in regards to the icon
        x -= popup.getContent().getWidth() / 2;

        popup.show(this, x, y);

        if (firstRender) {
            firstRender = false;
            show();
        }
    }

    /**
     * Hide the stream info popup.
     */
    public void hide() {
        popup.hide();
    }

    //endregion

    //region Functions

    private void init() {
        initializeEvents();

        getStyleClass().add(STYLE_CLASS);
    }

    private void initializeEvents() {
        setOnMouseEntered(event -> show());
        setOnMouseExited(event -> hide());
    }

    //endregion

    private class StreamPopup extends PopupControl {
        private final StreamPopupSkin skin = new StreamPopupSkin(this);

        @Override
        protected Skin<?> createDefaultSkin() {
            return skin;
        }

        /**
         * Get the content node of the popup.
         *
         * @return Returns the content node.
         */
        Pane getContent() {
            return (Pane) skin.getNode();
        }

        /**
         * Update the stream info in the popup.
         *
         * @param progress The update information to display.
         */
        void updateInfo(DownloadStatus progress) {
            Platform.runLater(() -> skin.updateInfo(progress));
        }
    }

    private class StreamPopupSkin implements Skin<StreamPopup> {
        private final StreamPopup streamPopup;

        private GridPane content;
        private StreamInfoCell download;
        private Label downloadValue;
        private StreamInfoCell upload;
        private Label uploadValue;
        private StreamInfoCell activePeers;
        private Label activePeersValue;
        private StreamInfoCell downloaded;
        private Label downloadedValue;

        private StreamPopupSkin(StreamPopup streamPopup) {
            this.streamPopup = streamPopup;

            init();
        }

        @Override
        public StreamPopup getSkinnable() {
            return streamPopup;
        }

        @Override
        public Node getNode() {
            return content;
        }

        @Override
        public void dispose() {
            this.content = null;
            this.download = null;
            this.downloadValue = null;
            this.upload = null;
            this.uploadValue = null;
            this.activePeers = null;
            this.activePeersValue = null;
            this.downloaded = null;
            this.downloadedValue = null;
        }

        void updateInfo(DownloadStatus DownloadStatus) {
            int progress = (int) (DownloadStatus.progress() * 100);
            String downloaded = MessageFormat.format("{0} ({1}%)", SizeUtils.toDisplaySize(DownloadStatus.downloaded()), progress);

            downloadValue.setText(SizeUtils.toDisplaySize(DownloadStatus.downloadSpeed()) + "/s");
            uploadValue.setText(SizeUtils.toDisplaySize(DownloadStatus.uploadSpeed()) + "/s");
            activePeersValue.setText(String.valueOf(DownloadStatus.connections()));
            downloadedValue.setText(downloaded);
        }

        private void init() {
            initializeContent();
            initializeDownload();
            initializeUpload();
            initializePeers();
            initializeDownloaded();
            initializeListeners();

            createCells();
        }

        private void initializeContent() {
            this.content = new GridPane();
            this.content.getStyleClass().add(POPUP_STYLE_CLASS);

            content.getColumnConstraints().addAll(createColumn(), createColumn());
            content.getRowConstraints().addAll(new RowConstraints(), new RowConstraints(), new RowConstraints(), new RowConstraints());
            content.setHgap(10);
        }

        private void initializeDownload() {
            this.downloadValue = new Label();
            this.downloadValue.getStyleClass().add(VALUE_STYLE_CLASS);

            content.add(downloadValue, 1, 0);
            GridPane.setHalignment(downloadValue, HPos.RIGHT);
        }

        private void initializeUpload() {
            this.uploadValue = new Label();
            this.uploadValue.getStyleClass().add(VALUE_STYLE_CLASS);

            content.add(uploadValue, 1, 1);
            GridPane.setHalignment(uploadValue, HPos.RIGHT);
        }

        private void initializePeers() {
            this.activePeersValue = new Label();
            this.activePeersValue.getStyleClass().add(VALUE_STYLE_CLASS);

            content.add(activePeersValue, 1, 2);
            GridPane.setHalignment(activePeersValue, HPos.RIGHT);
        }

        private void initializeDownloaded() {
            this.downloadedValue = new Label();
            this.downloadedValue.getStyleClass().add(VALUE_STYLE_CLASS);

            content.add(downloadedValue, 1, 3);
            GridPane.setHalignment(downloadedValue, HPos.RIGHT);
        }

        private void initializeListeners() {
            factoryProperty().addListener((observable, oldValue, newValue) -> Platform.runLater(this::createCells));
        }

        private void createCells() {
            // remove old cells if they exist
            if (this.download != null)
                content.getChildren().remove(this.download);
            if (this.upload != null)
                content.getChildren().remove(this.upload);
            if (this.activePeers != null)
                content.getChildren().remove(this.activePeers);
            if (this.downloaded != null)
                content.getChildren().remove(this.downloaded);

            // create new cells from the factory
            this.download = getFactory().apply(DOWNLOAD_CELL);
            this.upload = getFactory().apply(UPLOAD_CELL);
            this.activePeers = getFactory().apply(ACTIVE_PEERS_CELL);
            this.downloaded = getFactory().apply(DOWNLOADED_CELL);

            // add the new cells to the popup
            content.add(download, 0, 0);
            content.add(upload, 0, 1);
            content.add(activePeers, 0, 2);
            content.add(downloaded, 0, 3);
        }

        //TODO: cleanup
        private ColumnConstraints createColumn() {
            ColumnConstraints column = new ColumnConstraints();
//            column.setPercentWidth(50);
            return column;
        }
    }
}
