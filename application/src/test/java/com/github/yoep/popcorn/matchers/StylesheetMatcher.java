package com.github.yoep.popcorn.matchers;

import javafx.scene.Parent;
import org.hamcrest.Matcher;

import static org.testfx.matcher.base.GeneralMatchers.typeSafeMatcher;

public class StylesheetMatcher {
    private StylesheetMatcher() {
    }

    /**
     * Matcher which checks if the parent node contains the given stylesheet.
     *
     * @param stylesheet The stylesheet to match.
     * @return Returns the stylesheet matcher.
     */
    public static Matcher<Parent> hasStyleSheet(String stylesheet) {
        var descriptionText = "has stylesheet \"" + stylesheet + "\"";

        return typeSafeMatcher(Parent.class, descriptionText,
                parent -> "\"" + parent.getStylesheets() + "\"",
                parent -> parent.getStylesheets().contains(stylesheet));
    }
}
