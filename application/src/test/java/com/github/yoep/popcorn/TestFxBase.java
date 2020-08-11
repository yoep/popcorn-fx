package com.github.yoep.popcorn;

import com.github.yoep.popcorn.ui.PopcornTimeApplication;
import javafx.stage.Stage;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.springframework.test.context.junit.jupiter.SpringExtension;
import org.testfx.api.FxToolkit;
import org.testfx.framework.junit5.ApplicationTest;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeoutException;

@ExtendWith(SpringExtension.class)
public abstract class TestFxBase extends ApplicationTest {
    @BeforeAll
    public static void config() throws TimeoutException {
        System.setProperty("testfx.robot", "glass");
        // the ApplicationTest bypasses the main method in PopcornTimeApplication
        // so we need to manually provide the system property "app.dir" for testing purposes
        System.setProperty("app.dir", PopcornTimeApplication.APP_DIR);
        System.setProperty("jlibtorrent.jni.path", "/data/projects/popcorn-desktop-javafx/assets/linux/libjlibtorrent.so");

        FxToolkit.registerPrimaryStage();
        FxToolkit.setupApplication(PopcornTimeApplicationTest::new);
    }

    @BeforeEach
    void setUp() throws TimeoutException {
        FxToolkit.showStage();
        WaitForAsyncUtils.waitForFxEvents(100);
    }

    @Override
    public void start(Stage stage) {
        stage.toFront();
    }
}
