package com.github.yoep.popcorn.ui;

import javafx.scene.input.KeyCode;
import javafx.scene.input.MouseButton;
import javafx.stage.Stage;
import org.junit.After;
import org.junit.Before;
import org.junit.BeforeClass;
import org.testfx.api.FxToolkit;
import org.testfx.framework.junit.ApplicationTest;

import java.util.concurrent.TimeoutException;

public abstract class TestFxBase extends ApplicationTest {
    @BeforeClass
    public static void config() {
        System.getProperties().put("testfx.robot", "glass");
    }

    @Before
    public void setup() throws Exception {
        // the ApplicationTest bypasses the main method in PopcornTimeApplication
        // so we need to manually provide the system property "app.dir" for testing purposes
        System.setProperty("app.dir", PopcornTimeApplication.APP_DIR);
        System.setProperty("java.library.path", "");
        ApplicationTest.launch(PopcornTimeApplication.class);
    }

    @Override
    public void start(Stage stage) {
        stage.show();
    }

    @After
    public void afterEachTest() throws TimeoutException {
        FxToolkit.hideStage();
        release(new KeyCode[]{});
        release(new MouseButton[]{});
    }
}
