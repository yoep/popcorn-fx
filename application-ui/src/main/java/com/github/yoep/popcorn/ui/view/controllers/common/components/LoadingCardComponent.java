package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.Initializable;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Scope;

import java.net.URL;
import java.util.ResourceBundle;

import static org.springframework.beans.factory.config.BeanDefinition.SCOPE_PROTOTYPE;

@Slf4j
@Scope(SCOPE_PROTOTYPE)
@ViewController
public class LoadingCardComponent extends AbstractCardComponent implements Initializable {
    public LoadingCardComponent(ImageService imageService) {
        super(imageService);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePoster();
    }

    private void initializePoster() {
        setBackgroundImage(imageService.getPosterPlaceholder(POSTER_WIDTH, POSTER_HEIGHT), false);
    }
}
