module player.popcorn {
    requires application.backend;
    requires application.ui;
    requires com.google.protobuf;
    requires java.desktop;
    requires javafx.base;
    requires javafx.controls;
    requires javafx.fxml;
    requires javafx.graphics;
    requires org.apache.commons.io;
    requires org.slf4j;

    requires static lombok;

    exports com.github.yoep.player.popcorn.controllers.components;
    exports com.github.yoep.player.popcorn.controllers.sections;
    exports com.github.yoep.player.popcorn.player;
    exports com.github.yoep.player.popcorn.services;

    opens com.github.yoep.player.popcorn.controllers.components to javafx.fxml;
    opens com.github.yoep.player.popcorn.controllers.sections to javafx.fxml;
    opens com.github.yoep.player.popcorn.controls to javafx.fxml;

    opens views.common.popcorn.components;
    opens views.common.popcorn.sections;
    opens views.desktop.popcorn.components;
    opens views.tv.popcorn.components;
}