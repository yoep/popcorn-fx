package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.activities.ShowDetailsActivity;
import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.PlayMediaActivity;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.geometry.Insets;
import javafx.scene.effect.GaussianBlur;
import javafx.scene.image.Image;
import javafx.scene.layout.*;
import javafx.scene.paint.Color;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Scope;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;

import static org.springframework.beans.factory.config.BeanDefinition.SCOPE_PROTOTYPE;

@Slf4j
@Scope(SCOPE_PROTOTYPE)
@Component
@RequiredArgsConstructor
public class BackgroundComponent implements Initializable {
    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;

    @FXML
    private BorderPane backgroundImage;

    @PostConstruct
    private void init() {
        activityManager.register(ShowDetailsActivity.class, activity -> loadBackgroundImage(activity.getMedia()));
        activityManager.register(PlayMediaActivity.class, activity -> loadBackgroundImage(activity.getMedia()));
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        backgroundImage.setEffect(new GaussianBlur(30));
    }

    private void loadBackgroundImage(final Media media) {
        taskExecutor.execute(() -> {
            // set black background first
            Platform.runLater(() -> this.backgroundImage.setBackground(new Background(new BackgroundFill(Color.BLACK, CornerRadii.EMPTY, Insets.EMPTY))));

            // try to load the background image
            try {
                Optional.ofNullable(media.getImages())
                        .map(Images::getFanart)
                        .filter(e -> !e.equalsIgnoreCase("n/a"))
                        .map(Image::new)
                        .ifPresent(this::showBackgroundImage);
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });
    }

    private void showBackgroundImage(final Image image) {
        final BackgroundSize backgroundSize =
                new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, true, true);
        final BackgroundImage backgroundImage =
                new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.DEFAULT, backgroundSize);

        Platform.runLater(() -> this.backgroundImage.setBackground(new Background(backgroundImage)));
    }
}
