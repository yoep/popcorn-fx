package com.github.yoep.popcorn.subtitle.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
import javafx.collections.ObservableList;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Control;
import javafx.scene.control.Label;
import javafx.scene.control.Skin;
import javafx.scene.control.SkinBase;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.HBox;
import javafx.scene.layout.StackPane;
import lombok.Getter;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import java.io.IOException;

@Slf4j
public class LanguageFlagCell extends Control {
    private final CellSkin skin = new CellSkin(this);

    /**
     * Update the skin with the given item information.
     *
     * @param item The item to style the skin with.
     */
    public void updateItem(SubtitleInfo item) {
        if (item != null) {
            setText(item.getLanguage());

            item.getFlagResource().ifPresent(e -> {
                try {
                    setGraphic(new ImageView(new Image(e.getInputStream())));
                } catch (IOException ex) {
                    log.error(ex.getMessage(), ex);
                }
            });
        } else {
            setText(null);
            setGraphic(null);
        }
    }

    @Override
    protected Skin<?> createDefaultSkin() {
        return skin;
    }

    /**
     * Set the text of the skin.
     *
     * @param text The text to use in the skin.
     */
    protected void setText(String text) {
        Label label = skin.getLabel();
        ObservableList<Node> children = skin.getSkinNode().getChildren();

        label.setText(text);

        if (StringUtils.isEmpty(text)) {
            children.remove(label);
        } else if (!children.contains(label)) {
            children.add(1, label);
        }
    }

    /**
     * Set the graphic node of the skin.
     *
     * @param graphic The graphic node to use in the skin.
     */
    protected void setGraphic(Node graphic) {
        StackPane pane = skin.getGraphicPane();

        pane.getChildren().clear();
        pane.getChildren().add(graphic);
    }

    @Getter
    private static class CellSkin extends SkinBase<LanguageFlagCell> {
        private static final String GRAPHIC_STYLE_CLASS = "graphic";
        private static final String LABEL_STYLE_CLASS = "name";
        private static final String ARROW_STYLE_CLASS = "arrow";

        private HBox skinNode;
        private StackPane graphicPane;
        private Label label;
        private Icon arrow;

        /**
         * Constructor for all SkinBase instances.
         *
         * @param control The control for which this Skin should attach to.
         */
        protected CellSkin(LanguageFlagCell control) {
            super(control);
            init();
        }

        @Override
        public void dispose() {
            graphicPane = null;
            label = null;
            arrow = null;
            skinNode = null;
        }

        private void init() {
            initializeNode();
            initializeGraphic();
            initializeLabel();
            initializeArrow();
        }

        private void initializeNode() {
            skinNode = new HBox();
            skinNode.setAlignment(Pos.CENTER);

            getChildren().add(skinNode);
        }

        private void initializeGraphic() {
            graphicPane = new StackPane();
            graphicPane.getStyleClass().add(GRAPHIC_STYLE_CLASS);

            skinNode.getChildren().add(graphicPane);
        }

        private void initializeLabel() {
            label = new Label();
            label.getStyleClass().add(LABEL_STYLE_CLASS);
            label.setMaxWidth(Integer.MAX_VALUE);

            skinNode.getChildren().add(label);
        }

        private void initializeArrow() {
            arrow = new Icon(Icon.CARET_UP_UNICODE);
            arrow.getStyleClass().add(ARROW_STYLE_CLASS);

            skinNode.getChildren().add(arrow);
        }
    }
}
