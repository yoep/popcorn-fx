package com.github.yoep.popcorn;

import com.github.yoep.popcorn.backend.settings.SettingsDefaults;
import javafx.stage.Stage;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.testfx.api.FxToolkit;
import org.testfx.framework.junit5.ApplicationTest;
import org.testfx.util.WaitForAsyncUtils;

import java.io.File;
import java.util.concurrent.TimeoutException;

public abstract class TestFxBase extends ApplicationTest {
    protected PopcornTimeApplicationTest application;

    @BeforeAll
    public static void config() {
        System.setProperty("testfx.robot", "glass");
        // the ApplicationTest bypasses the main method in PopcornTimeApplication
        // so we need to manually provide the system property "app.dir" for testing purposes
        System.setProperty("app.dir", SettingsDefaults.APP_DIR);

        initializeLibTorrentPath();
    }

    @BeforeEach
    public void setUp() throws TimeoutException {
        FxToolkit.registerStage(Stage::new);
        application = (PopcornTimeApplicationTest) FxToolkit.setupApplication(PopcornTimeApplicationTest.class, "--disable-popcorn-keys");
        FxToolkit.showStage();
        WaitForAsyncUtils.waitForFxEvents(100);
    }

    @AfterEach
    public void tearDown() throws TimeoutException {
        FxToolkit.cleanupStages();
        FxToolkit.cleanupApplication(application);
    }

    @Override
    public void start(Stage stage) {
        stage.toFront();
    }

    private static void initializeLibTorrentPath() {
        File applicationPath;
        File rootPath;
        String path;

        if (System.getProperty("os.name").contains("Windows")) {
            applicationPath = new File("../assets/windows/jlibtorrent.dll");
            rootPath = new File("assets/windows/jlibtorrent.dll");
        } else if (System.getProperty("os.name").startsWith("Mac OS")) {
            applicationPath = new File("../assets/macosx/libjlibtorrent.dylib");
            rootPath = new File("assets/macosx/libjlibtorrent.dylib");
        } else {
            applicationPath = new File("../assets/linux/libjlibtorrent.so");
            rootPath = new File("assets/linux/libjlibtorrent.so");
        }

        if (applicationPath.exists()) {
            path = applicationPath.getAbsolutePath();
        } else {
            path = rootPath.getAbsolutePath();
        }

        System.setProperty("jlibtorrent.jni.path", path);
    }
}
