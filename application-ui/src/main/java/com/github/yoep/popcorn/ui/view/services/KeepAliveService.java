package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.ui.settings.OptionsService;
import javafx.scene.input.KeyCode;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Service;

/**
 * Service which will keep the screen and machine alive by sending random inputs to the system.
 * This will prevent the screen from blanking and the machine from going to standby.
 * <p>
 * It uses a {@link Scheduled} task in the background which is managed by the Spring Framework.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class KeepAliveService {
    public static final KeyCode SIGNAL = KeyCode.CONTROL;

    private final OptionsService optionsService;
    private final RobotService robotService;

    @Scheduled(fixedRate = 3 * 60 * 1000, initialDelay = 5 * 60 * 1000)
    public void keepAlive() {
        // check if the keep alive service should be disabled
        // if so, ignore the invocations
        if (isDisabled())
            return;

        robotService.pressKey(SIGNAL);
    }

    private boolean isDisabled() {
        return optionsService.options().isKeepAliveDisabled();
    }
}
