package com.github.yoep.popcorn;

import com.github.spring.boot.javafx.SpringJavaFXApplication;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.spring.boot.javafx.view.ViewManagerPolicy;
import com.github.spring.boot.javafx.view.ViewProperties;
import javafx.stage.Stage;
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication
public class PopcornTimeApplication extends SpringJavaFXApplication {

    public static void main(String[] args) {
        launch(PopcornTimeApplication.class, PopcornTimePreloader.class, args);
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        super.start(primaryStage);
        ViewLoader loader = applicationContext.getBean(ViewLoader.class);
        ViewManager viewManager = applicationContext.getBean(ViewManager.class);

        loader.show(primaryStage, "main.fxml", ViewProperties.builder()
                .title("Popcorn Time")
                .icon("icon.png")
                .centerOnScreen(true)
                .maximizable(true)
                .build());
        viewManager.setPolicy(ViewManagerPolicy.CLOSEABLE);
    }
}
