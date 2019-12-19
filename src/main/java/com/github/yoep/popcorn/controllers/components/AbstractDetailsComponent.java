package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.controls.Stars;
import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.Pane;
import javafx.util.Duration;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.task.TaskExecutor;

import java.util.Optional;

/**
 * Abstract definition of a details component for showing {@link Media} information.
 *
 * @param <T> The media type of the details component.
 */
@Slf4j
public abstract class AbstractDetailsComponent<T extends Media> implements Initializable {
    protected final TaskExecutor taskExecutor;

    protected T media;

    @FXML
    protected Pane posterHolder;
    @FXML
    protected ImageView poster;
    @FXML
    protected Stars ratingStars;

    public AbstractDetailsComponent(TaskExecutor taskExecutor) {
        this.taskExecutor = taskExecutor;
    }

    protected void initializePoster() {
        poster.fitHeightProperty().bind(posterHolder.heightProperty());
        poster.fitWidthProperty().bind(posterHolder.widthProperty());
    }

    protected void loadStars() {
        ratingStars.setRating(media.getRating());
        Tooltip tooltip = new Tooltip(media.getRating().getPercentage() / 10 + "/10");
        setInstantTooltip(tooltip);
        Tooltip.install(ratingStars, tooltip);
    }

    protected void loadPosterImage() {
        // load the poster image in the background
        taskExecutor.execute(() -> {
            try {
                final Image posterImage = Optional.ofNullable(media.getImages())
                        .map(Images::getPoster)
                        .filter(e -> !e.equalsIgnoreCase("n/a"))
                        .map(Image::new)
                        .orElse(new Image(new ClassPathResource("/images/posterholder.png").getInputStream()));

                Platform.runLater(() -> poster.setImage(posterImage));
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });
    }

    protected void setInstantTooltip(Tooltip tooltip) {
        tooltip.setShowDelay(Duration.ZERO);
        tooltip.setShowDuration(Duration.INDEFINITE);
        tooltip.setHideDelay(Duration.ZERO);
    }
}
