package com.github.yoep.popcorn.matchers;

import javafx.css.Styleable;
import javafx.scene.Parent;
import org.hamcrest.Matcher;

import static org.testfx.matcher.base.GeneralMatchers.typeSafeMatcher;

public class StyleClassMatchers {
    private StyleClassMatchers() {
    }

    /**
     * Matcher which checks if the styleable class contains the given style class.
     *
     * @param styleClass The style class to match.
     * @return Returns the style class matcher.
     */
    public static Matcher<Styleable> hasStyleClass(String styleClass) {
        var descriptionText = "has style class \"" + styleClass + "\"";

        return typeSafeMatcher(Parent.class, descriptionText,
                parent -> "\"" + parent.getStyleClass() + "\"",
                parent -> parent.getStyleClass().contains(styleClass));
    }
}
