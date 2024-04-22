package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.info.ComponentInfo;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import javafx.application.Platform;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.ColumnConstraints;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Priority;
import javafx.scene.layout.RowConstraints;

public class AboutCard extends GridPane {
    public static final String EXPANDED_PROPERTY = "expanded";
    private static final String GRID_STYLE_CLASS = "details-card";
    private static final String CARET_STYLE_CLASS = "caret";
    private static final String NAME_STYLE_CLASS = "name";
    private static final String DESCRIPTION_STYLE_CLASS = "description";
    private static final String STATE_STYLE_CLASS = "state";

    private final ComponentInfo componentInfo;
    final Icon caretIcon = new Icon(Icon.CARET_RIGHT_UNICODE);
    final Label nameLabel = new Label();
    final Label descriptionLabel = new Label();
    final Icon stateIcon = new Icon();

    private final BooleanProperty expanded = new SimpleBooleanProperty(this, EXPANDED_PROPERTY);

    public AboutCard(ComponentInfo componentInfo) {
        this.componentInfo = componentInfo;
        init();
    }

    //region Properties

    public boolean getExpanded() {
        return expanded.get();
    }

    public BooleanProperty expandedProperty() {
        return expanded;
    }

    public void setExpanded(boolean expanded) {
        this.expanded.set(expanded);
    }

    //endregion

    //region Initialize

    private void init() {
        initializeGrid();
        initializeCaret();
        initializeName();
        initializeDescription();
        initializeState();
        initializeProperties();

        add(caretIcon, 0, 0);
        add(nameLabel, 1, 0);
        add(stateIcon, 2, 0);
        add(descriptionLabel, 1, 1);
    }

    private void initializeCaret() {
        caretIcon.getStyleClass().add(CARET_STYLE_CLASS);
        caretIcon.setSizeFactor(1.5);
        caretIcon.setOnMouseClicked(this::onTitleRowClicked);
    }

    private void initializeName() {
        nameLabel.setText(componentInfo.getName());
        nameLabel.getStyleClass().add(NAME_STYLE_CLASS);
        nameLabel.setMaxWidth(Double.MAX_VALUE);
        nameLabel.setOnMouseClicked(this::onTitleRowClicked);
    }

    private void initializeDescription() {
        descriptionLabel.setText(componentInfo.getDescription().orElse(null));
        descriptionLabel.getStyleClass().add(DESCRIPTION_STYLE_CLASS);
    }

    private void initializeState() {
        stateIcon.setText(stateToIconUnicode(componentInfo.getState()));
        stateIcon.setSizeFactor(1.5);
        stateIcon.getStyleClass().add(STATE_STYLE_CLASS);

        componentInfo.addChangeListener(evt -> {
            if (evt.getPropertyName().equals(ComponentInfo.STATE_PROPERTY)) {
                Platform.runLater(() -> stateIcon.setText(stateToIconUnicode((ComponentState) evt.getNewValue())));
            }
        });
    }

    private void initializeProperties() {
        expanded.addListener((observable, oldValue, newValue) ->
                caretIcon.setText(newValue ? Icon.CARET_DOWN_UNICODE : Icon.CARET_RIGHT_UNICODE));
    }

    private void initializeGrid() {
        var caretColumn = new ColumnConstraints();
        var detailsColumn = new ColumnConstraints();
        var stateColumn = new ColumnConstraints();
        var titleRow = new RowConstraints();
        var summaryRow = new RowConstraints();

        caretColumn.setPrefWidth(20);
        detailsColumn.setHgrow(Priority.ALWAYS);

        getStyleClass().add(GRID_STYLE_CLASS);
        getColumnConstraints().addAll(caretColumn, detailsColumn, stateColumn);
        getRowConstraints().addAll(titleRow, summaryRow);
    }

    //endregion

    private void onTitleRowClicked(MouseEvent event) {
        event.consume();
        setExpanded(!getExpanded());
    }

    private static String stateToIconUnicode(ComponentState state) {
        switch (state) {
            case UNKNOWN:
                return Icon.QUESTION_CIRCLE_UNICODE;
            case READY:
                return Icon.CHECK_CIRCLE_UNICODE;
            case ERROR:
            default:
                return Icon.TIMES_CIRCLE_O_UNICODE;
        }
    }
}
