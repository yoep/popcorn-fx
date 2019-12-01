package org.github.popcorn;

import com.sun.javafx.application.LauncherImpl;
import javafx.application.Application;
import javafx.stage.Stage;
import org.github.popcorn.ui.ViewLoader;
import org.github.popcorn.ui.ViewManager;
import org.github.popcorn.ui.ViewManagerPolicy;
import org.github.popcorn.ui.ViewProperties;
import org.springframework.boot.Banner;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.ApplicationContext;

@SpringBootApplication
public class PopcornTimeApplication extends Application {
    public static ApplicationContext APPLICATION_CONTEXT;

    private static String[] ARGUMENTS;

    public static void main(String[] args) {
        ARGUMENTS = args;

        LauncherImpl.launchApplication(PopcornTimeApplication.class, args);
    }

    @Override
    public void init() {
        SpringApplication application = new SpringApplication(PopcornTimeApplication.class);
        application.setBannerMode(Banner.Mode.OFF);
        APPLICATION_CONTEXT = application.run(ARGUMENTS);
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        ViewLoader loader = APPLICATION_CONTEXT.getBean(ViewLoader.class);
        ViewManager viewManager = APPLICATION_CONTEXT.getBean(ViewManager.class);

        loader.show(primaryStage, "main.fxml", ViewProperties.builder()
                .title("Popcorn Time")
                .icon("icon.png")
                .centerOnScreen(true)
                .maximizable(true)
                .build());
        viewManager.setPolicy(ViewManagerPolicy.CLOSEABLE);
    }
}
