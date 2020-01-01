package com.github.yoep.popcorn.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.media.providers.models.Rating;
import javafx.geometry.Pos;
import javafx.scene.control.OverrunStyle;
import javafx.scene.layout.HBox;
import javafx.scene.layout.StackPane;
import org.springframework.util.Assert;

public class Stars extends HBox {
    private Rating rating;

    public Stars() {
        getStyleClass().add("stars");
    }

    public Stars(Rating rating) {
        Assert.notNull(rating, "rating cannot be null");
        this.rating = rating;
        createStars();
        getStyleClass().add("stars");
    }

    /**
     * Set the rating that the stars must represent.
     *
     * @param rating The rating to represent.
     */
    public void setRating(Rating rating) {
        Assert.notNull(rating, "rating cannot be null");
        this.rating = rating;
        createStars();
    }

    /**
     * Reset the stars.
     */
    public void reset() {
        this.getChildren().clear();
    }

    private void createStars() {
        reset();

        double rating = this.rating.getPercentage() * 0.05;

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
        star.getStyleClass().add("filled");
        return star;
    }

    private static Icon getHalfStar() {
        Icon star = new Icon(Icon.STAR_HALF_UNICODE);
        star.setTextOverrun(OverrunStyle.CLIP);
        star.getStyleClass().add("filled");
        return star;
    }

    private static Icon getEmptyStar() {
        Icon star = new Icon(Icon.STAR_UNICODE);
        star.setTextOverrun(OverrunStyle.CLIP);
        star.getStyleClass().add("empty");
        return star;
    }
}
