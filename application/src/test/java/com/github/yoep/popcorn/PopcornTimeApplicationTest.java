package com.github.yoep.popcorn;

import com.github.spring.boot.javafx.view.ViewManager;
import com.github.spring.boot.javafx.view.ViewManagerPolicy;
import com.github.yoep.popcorn.ui.PopcornTimeApplication;
import javafx.stage.Stage;
import org.springframework.context.annotation.Primary;

@Primary
public class PopcornTimeApplicationTest extends PopcornTimeApplication {
    @Override
    public void start(Stage stage) throws Exception {
        super.start(stage);
        var viewManager = applicationContext.getBean(ViewManager.class);

        viewManager.setPolicy(ViewManagerPolicy.BLOCKED);
    }
}
