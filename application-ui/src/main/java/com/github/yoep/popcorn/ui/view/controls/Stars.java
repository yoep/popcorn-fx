package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.media.providers.models.Rating;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import javafx.geometry.Pos;
import javafx.scene.control.OverrunStyle;
import javafx.scene.layout.HBox;
import javafx.scene.layout.StackPane;

import java.util.Objects;

public class Stars extends HBox {
    private static final String STARS_STYLE_CLASS = "stars";
    private static final String STAR_STYLE_CLASS = "star";

    private Rating rating;

    //region Constructors

    public Stars() {
        init();
    }

    public Stars(Rating rating) {
        Objects.requireNonNull(rating, "rating cannot be null");
        this.rating = rating;
        init();
    }

    //endregion

    //region Methods

    /**
     * Set the rating that the stars must represent.
     *
     * @param rating The rating to represent.
     */
    public void setRating(Rating rating) {
        Objects.requireNonNull(rating, "rating cannot be null");
        this.rating = rating;
        initializeStars();
    }

    /**
     * Reset the stars.
     */
    public void reset() {
        this.getChildren().clear();
    }

    //endregion

    //region Functions

    private void init() {
        initializeStyleClass();
        initializeStars();
    }

    private void initializeStyleClass() {
        getStyleClass().add(STARS_STYLE_CLASS);
    }

    private void initializeStars() {
        reset();

        // check if a rating is present
        if (rating == null)
            return;

        var rating = this.rating.getPercentage() * 0.05;

        for (int i = 0; i < 5; i++) {
            if (rating >= 0.75) {
                getChildren().add(getFilledStar());
            } else if (rating < 0.75 && rating > 0.25) {
                StackPane stackpane = new StackPane(getEmptyStar(), getHalfStar());
                stackpane.setAlignment(Pos.CENTER_LEFT);
                getChildren().add(stackpane);
            } else {
                getChildren().add(getEmptyStar());
            }

            rating = --rating;
        }
    }

    private static Icon getFilledStar() {
        Icon star = new Icon(Icon.STAR_UNICODE);
        star.setTextOverrun(OverrunStyle.CLIP);
        star.getStyleClass().addAll(STAR_STYLE_CLASS, "filled");
        return star;
    }

    private static Icon getHalfStar() {
        Icon star = new Icon(Icon.STAR_HALF_UNICODE);
        star.setTextOverrun(OverrunStyle.CLIP);
        star.getStyleClass().addAll(STAR_STYLE_CLASS, "filled");
        return star;
    }

    private static Icon getEmptyStar() {
        Icon star = new Icon(Icon.STAR_UNICODE);
        star.setTextOverrun(OverrunStyle.CLIP);
        star.getStyleClass().addAll(STAR_STYLE_CLASS, "empty");
        return star;
    }

    //endregion
}
