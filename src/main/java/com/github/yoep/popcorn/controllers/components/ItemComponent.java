package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.providers.media.models.Images;
import com.github.yoep.popcorn.providers.media.models.Media;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;

import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class ItemComponent implements Initializable {
    private final Media media;

    @FXML
    private ImageView poster;
    @FXML
    private Label title;
    @FXML
    private Label year;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeImage();
        initializeText();
    }

    private void initializeImage() {
        try {
            Image image = Optional.ofNullable(media.getImages())
                    .map(Images::getPoster)
                    .filter(e -> !e.equalsIgnoreCase("n/a"))
                    .map(Image::new)
                    .orElse(new Image(new ClassPathResource("/images/posterholder.png").getInputStream()));

            poster.setImage(image);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void initializeText() {
        title.setText(media.getTitle());
        year.setText(media.getYear());

        Tooltip.install(title, new Tooltip(media.getTitle()));
    }
}
