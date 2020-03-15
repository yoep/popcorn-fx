package com.github.yoep.popcorn.view.controllers.desktop.components;

import javafx.fxml.Initializable;
import javafx.scene.image.Image;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

import static org.springframework.beans.factory.config.BeanDefinition.SCOPE_PROTOTYPE;

@Slf4j
@Scope(SCOPE_PROTOTYPE)
@Component
@RequiredArgsConstructor
public class LoadingCardComponent extends AbstractCardComponent implements Initializable {
    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePoster();
    }

    private void initializePoster() {
        try {
            Image image = new Image(getPosterHolderResource().getInputStream(), POSTER_WIDTH, POSTER_HEIGHT, true, true);
            setBackgroundImage(image, false);
        } catch (IOException ex) {
            log.error(ex.getMessage(), ex);
        }
    }

}
