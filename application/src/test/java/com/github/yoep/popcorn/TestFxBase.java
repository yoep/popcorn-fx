package com.github.yoep.popcorn;

import com.github.yoep.popcorn.ui.PopcornTimeApplication;
import javafx.application.Application;
import javafx.stage.Stage;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.springframework.test.context.junit.jupiter.SpringExtension;
import org.testfx.api.FxToolkit;
import org.testfx.framework.junit5.ApplicationTest;
import org.testfx.util.WaitForAsyncUtils;

import java.io.File;
import java.util.concurrent.TimeoutException;

@ExtendWith(SpringExtension.class)
public abstract class TestFxBase extends ApplicationTest {
    protected Application application;

    @BeforeAll
    public static void config() {
        System.setProperty("testfx.robot", "glass");
        // the ApplicationTest bypasses the main method in PopcornTimeApplication
        // so we need to manually provide the system property "app.dir" for testing purposes
        System.setProperty("app.dir", PopcornTimeApplication.APP_DIR);

        initializeLibTorrentPath();
    }

    @BeforeEach
    void setUp() throws TimeoutException {
        FxToolkit.registerStage(Stage::new);
        FxToolkit.setupApplication(PopcornTimeApplicationTest.class);
        FxToolkit.showStage();
        WaitForAsyncUtils.waitForFxEvents(100);
    }

    @AfterEach
    void tearDown() throws TimeoutException {
        FxToolkit.cleanupStages();
        FxToolkit.cleanupApplication(application);
    }

    @Override
    public void start(Stage stage) {
        stage.toFront();
    }

    private static void initializeLibTorrentPath() {
        if (System.getProperty("os.arch").equals("amd64")) {
            String path;
            var applicationPath = new File("../assets/linux/libjlibtorrent.so");
            var rootPath = new File("assets/linux/libjlibtorrent.so");

            if (applicationPath.exists()) {
                path = applicationPath.getAbsolutePath();
            } else {
                path = rootPath.getAbsolutePath();
            }

            System.setProperty("jlibtorrent.jni.path", path);
        }
    }
}
