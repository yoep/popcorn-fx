import com.github.yoep.popcorn.backend.logging.LoggingBridge;

module application.ui {
    requires application.backend;
    requires java.datatransfer;
    requires java.desktop;
    requires javafx.base;
    requires javafx.controls;
    requires javafx.fxml;
    requires javafx.graphics;
    requires javafx.web;
    requires org.apache.commons.io;
    requires org.slf4j;

    requires static lombok;
    requires com.google.protobuf;

    uses LoggingBridge;

    exports com.github.yoep.popcorn.ui.events;
    exports com.github.yoep.popcorn.ui.font.controls;
    exports com.github.yoep.popcorn.ui.font;
    exports com.github.yoep.popcorn.ui.view.controls;
    exports com.github.yoep.popcorn.ui.view.services;
    exports com.github.yoep.popcorn.ui.view;
    exports com.github.yoep.popcorn.ui;
    exports com.github.yoep.popcorn.ui.view.controllers.common.components;

    opens com.github.yoep.popcorn.ui.torrent.controls to javafx.fxml;
    opens com.github.yoep.popcorn.ui.view.controllers to javafx.fxml;
    opens com.github.yoep.popcorn.ui.view.controllers.common.components to javafx.fxml;
    opens com.github.yoep.popcorn.ui.view.controllers.common.sections to javafx.fxml;
    opens com.github.yoep.popcorn.ui.view.controllers.desktop.components to javafx.fxml;
    opens com.github.yoep.popcorn.ui.view.controllers.desktop.sections to javafx.fxml;
    opens com.github.yoep.popcorn.ui.view.controllers.tv.components to javafx.fxml;
    opens com.github.yoep.popcorn.ui.view.controls to javafx.fxml;
    opens fonts to javafx.fxml;
    opens images.flags to javafx.fxml;

    opens images.icons;
    opens images.windows;
    opens images;
    opens styles;
    opens views.common.components;
    opens views.common.sections;
    opens views.desktop.components;
    opens views.desktop.sections;
    opens views.tv.components;
    opens views.tv.sections;
    opens views;
}