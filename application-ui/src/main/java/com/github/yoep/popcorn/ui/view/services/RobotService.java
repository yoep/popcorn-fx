package com.github.yoep.popcorn.ui.view.services;

import javafx.scene.input.KeyCode;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.awt.*;

@Slf4j
@Service
@NoArgsConstructor
public class RobotService {
    private Robot robot;

    RobotService(Robot robot) {
        this.robot = robot;
    }

    /**
     * Press the given key keyCode.
     * This will simulate a keyboard stroke for the given {@link KeyCode}.
     * The key press might not be invoked if the {@link Robot} failed to initialize.
     *
     * @param keyCode The keyCode to press.
     */
    public void pressKey(KeyCode keyCode) {
        Assert.notNull(keyCode, "keyCode cannot be null");
        if (isEnabled()) {
            var key = keyCode.getCode();

            robot.keyPress(key);
            robot.keyRelease(key);
        }
    }

    @PostConstruct
    private void init() {
        try {
            robot = new Robot();
        } catch (AWTException ex) {
            log.error("Robot service could not be enabled, " + ex.getMessage(), ex);
        }
    }

    private boolean isEnabled() {
        return robot != null;
    }
}
