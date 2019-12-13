package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.controls.Stars;
import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.layout.*;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.DisposableBean;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.task.TaskExecutor;
import org.springframework.util.Assert;

import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
public class MediaCardComponent implements Initializable, DisposableBean {
    private static final int POSTER_WIDTH = 134;
    private static final int POSTER_HEIGHT = 196;

    private final List<ItemListener> listeners = new ArrayList<>();
    private final Media media;
    private final TaskExecutor taskExecutor;

    private Thread imageLoadingThread;

    @FXML
    private BorderPane poster;
    @FXML
    private Label title;
    @FXML
    private Label year;
    @FXML
    private Label ratingValue;
    @FXML
    private Stars ratingStars;

    public MediaCardComponent(Media media, TaskExecutor taskExecutor, ItemListener... listeners) {
        this.media = media;
        this.taskExecutor = taskExecutor;
        this.listeners.addAll(asList(listeners));
    }

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeImage();
        initializeText();
        initializeStars();
    }

    /**
     * Add a listener to this instance.
     *
     * @param listener The listener to add.
     */
    public void addListener(ItemListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    @Override
    public void destroy() {
        Optional.ofNullable(imageLoadingThread)
                .ifPresent(Thread::interrupt);
    }

    private void initializeImage() {
        imageLoadingThread = new Thread(() -> {
            try {
                // set as default the poster holder image
                setBackgroundImage(new Image(new ClassPathResource("/images/posterholder.png").getInputStream(), POSTER_WIDTH, POSTER_HEIGHT, true, true));

                //try to load the actual image
                Optional.ofNullable(media.getImages())
                        .map(Images::getPoster)
                        .filter(e -> !e.equalsIgnoreCase("n/a"))
                        .ifPresent(e -> setBackgroundImage(new Image(e, POSTER_WIDTH, POSTER_HEIGHT, true, true)));
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });

        // run this on a separate thread for easier UI loading
        taskExecutor.execute(imageLoadingThread);
    }

    private void setBackgroundImage(Image image) {
        BackgroundSize size = new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, false, true);
        BackgroundImage backgroundImage = new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.CENTER, size);
        poster.setBackground(new Background(backgroundImage));
    }

    private void initializeText() {
        double rating = (double) media.getRating().getPercentage() / 10;

        title.setText(media.getTitle());
        year.setText(media.getYear());
        ratingValue.setText(rating + "/10");

        Tooltip.install(title, new Tooltip(media.getTitle()));
    }

    private void initializeStars() {
        ratingStars.setRating(media.getRating());
    }

    @FXML
    private void showDetails() {
        synchronized (listeners) {
            listeners.forEach(e -> e.onClicked(media));
        }
    }
}