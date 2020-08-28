package com.github.yoep.popcorn.ui.view.services;

import javafx.scene.input.KeyCode;
import lombok.extern.slf4j.Slf4j;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Service;

import java.awt.*;

/**
 * Service which will keep the screen and machine alive by sending random inputs to the system.
 * This will prevent the screen from blanking and the machine from going to standby.
 *
 * It uses a {@link Scheduled} task in the background which is managed by the Spring Framework.
 */
@Slf4j
@Service
public class KeepAliveService {
    @Scheduled(fixedRate = 3 * 60 * 1000, initialDelay = 5 * 60 * 1000)
    public void keepAlive() {
        try {
            var robot = new Robot();
            var key = KeyCode.ALT.getCode();

            robot.keyPress(key);
            robot.keyRelease(key);
        } catch (AWTException ex) {
            log.error(ex.getMessage(), ex);
        }
    }
}