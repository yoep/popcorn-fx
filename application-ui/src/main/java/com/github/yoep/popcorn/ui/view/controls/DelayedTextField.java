package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.property.*;
import javafx.scene.control.TextField;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import lombok.extern.slf4j.Slf4j;

import java.awt.*;

/**
 * The {@link DelayedTextField} invokes the value changed event after the last input event on the {@link TextField} to prevent event stacking.
 */
@Slf4j
public class DelayedTextField extends TextField {
    public static final String INVOCATION_DELAY_PROPERTY = "invocationDelay";
    public static final String INPUT_DELAY_PROPERTY = "inputDelay";
    public static final String VALUE_PROPERTY = "value";
    private static final int MILLIS_BETWEEN_INVOKES = 300;
    private static final int MILLIS_AFTER_USER_INPUT = 300;
    private static final int WATCHER_TTL = 5000;

    /**
     * The minimum time between each invocation.
     */
    private final IntegerProperty invocationDelay = new SimpleIntegerProperty(this, INVOCATION_DELAY_PROPERTY, MILLIS_BETWEEN_INVOKES);
    private final IntegerProperty userDelay = new SimpleIntegerProperty(this, INPUT_DELAY_PROPERTY, MILLIS_AFTER_USER_INPUT);
    private final StringProperty value = new SimpleStringProperty(this, VALUE_PROPERTY);

    private boolean keepWatcherAlive;
    private boolean ignoreInvocation;
    private long lastChangeInvoked;
    private long lastUserInput;

    //region Constructors

    /**
     * Initialize a new instance of the {@link DelayedTextField}.
     */
    public DelayedTextField() {
        super("");
        init();
    }

    /**
     * Initialize a new instance of the {@link DelayedTextField}.
     *
     * @param text The initial value of the delayed text field.
     */
    public DelayedTextField(String text) {
        super(text);
        init();
    }

    //endregion

    //region Properties

    /**
     * Get the current invocation delay of the delayed text field.
     * Use this method for the delayed invocation instead of the {@link #getText()}.
     *
     * @return Returns the invocation delay.
     */
    public int getInvocationDelay() {
        return invocationDelay.get();
    }

    /**
     * Get the invocation delay property.
     *
     * @return Returns the delay property.
     */
    public ReadOnlyIntegerProperty invocationDelayProperty() {
        return invocationDelay;
    }

    /**
     * Set the new invocation delay.
     *
     * @param invocationDelay The delay between invocations.
     */
    public void setInvocationDelay(int invocationDelay) {
        if (invocationDelay < 0)
            throw new IllegalArgumentException("delay cannot be smaller than 0");

        this.invocationDelay.set(invocationDelay);
    }

    /**
     * Get the current delay between the last user input and invocation of the value.
     *
     * @return Returns the user delay of the text field.
     */
    public int getUserDelay() {
        return userDelay.get();
    }

    /**
     * Get the property of the last user input delay.
     *
     * @return Returns the user delay input property.
     */
    public ReadOnlyIntegerProperty userDelayProperty() {
        return userDelay;
    }

    /**
     * Set the new value for the last user input delay and actual invocation.
     *
     * @param userDelay The user input delay invocation.
     */
    public void setUserDelay(int userDelay) {
        if (userDelay < 0)
            throw new IllegalArgumentException("userDelay cannot be smaller than 0");

        this.userDelay.set(userDelay);
    }

    /**
     * Get the current value of the delayed text field.
     *
     * @return Returns the current value.
     */
    public String getValue() {
        return value.get();
    }

    /**
     * Get the value property of the delayed text field.
     *
     * @return Returns the value property.
     */
    public ReadOnlyStringProperty valueProperty() {
        return value;
    }

    /**
     * Set the new value property of the delayed text field.
     * Use this method instead of the text property to prevent the value change invocation.
     *
     * @param value The new delayed text field value.
     */
    public void setValue(String value) {
        ignoreInvocation = true;
        setText(value);
    }

    //endregion

    //region Methods

    @Override
    public void clear() {
        super.clear();
        onChanged();
    }

    //endregion

    //region Functions

    private void init() {
        initializeValue();
        initializeListeners();
        initializeActionListener();
        initializeKeyEvents();
    }

    private void initializeValue() {
        setValue(getText());
    }

    private void initializeListeners() {
        textProperty().addListener((observable, oldValue, newValue) -> {
            if (ignoreInvocation) {
                ignoreInvocation = false;
                return;
            }

            lastUserInput = System.currentTimeMillis();

            if (!keepWatcherAlive)
                createWatcher();
        });
    }

    private void initializeActionListener() {
        // if the ENTER key is pressed
        // force the value invocation
        setOnAction(event -> onChanged());
    }

    private void initializeKeyEvents() {
        try {
            var robot = new Robot();
            var focusMoveCode = KeyCode.TAB.getCode();
            var previousCode = KeyCode.SHIFT.getCode();

            this.addEventHandler(KeyEvent.KEY_PRESSED, event -> {
                if (event.getCode() == KeyCode.DOWN) {
                    event.consume();

                    robot.keyPress(focusMoveCode);
                    robot.keyRelease(focusMoveCode);
                } else if (event.getCode() == KeyCode.UP) {
                    event.consume();

                    robot.keyPress(previousCode);
                    robot.keyPress(focusMoveCode);
                    robot.keyRelease(previousCode);
                    robot.keyRelease(focusMoveCode);
                }
            });
        } catch (AWTException ex) {
            log.error("Failed to create episodes robot, " + ex.getMessage(), ex);
        }
    }

    private void createWatcher() {
        keepWatcherAlive = true;

        runTask(() -> {
            try {
                while (keepWatcherAlive) {
                    if (isInvocationAllowed()) {
                        onChanged();
                    }

                    // stop the watcher if the last user interaction was more than #WATCHER_TTL millis ago
                    if (System.currentTimeMillis() - lastUserInput > WATCHER_TTL)
                        keepWatcherAlive = false;

                    Thread.sleep(100);
                }
            } catch (InterruptedException ex) {
                log.error(ex.getMessage(), ex);
            }
        });
    }

    private void onChanged() {
        lastChangeInvoked = System.currentTimeMillis();
        value.set(getText());
    }

    private boolean isInvocationAllowed() {
        var currentTimeMillis = System.currentTimeMillis();

        return currentTimeMillis - lastUserInput > getUserDelay() &&
                currentTimeMillis - lastChangeInvoked > getInvocationDelay();
    }

    private void runTask(Runnable task) {
        new Thread(task, "DelayedTextField").start();
    }

    //endregion
}
