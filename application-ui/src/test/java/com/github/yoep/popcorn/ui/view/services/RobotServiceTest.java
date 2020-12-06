package com.github.yoep.popcorn.ui.view.services;

import javafx.scene.input.KeyCode;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.awt.*;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class RobotServiceTest {
    @Mock
    private Robot robot;

    private RobotService robotService;

    @BeforeEach
    void setUp() {
        robotService = new RobotService(robot);
    }

    @Test
    void testPressKey_whenKeyCodeIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> robotService.pressKey(null), "keyCode cannot be null");
    }

    @Test
    void testPressKey_whenRobotIsPresent_shouldInvokedKeyPressAndRelease() {
        var keyCode = KeyCode.ENTER;
        var code = keyCode.getCode();

        robotService.pressKey(keyCode);

        verify(robot).keyPress(code);
        verify(robot).keyRelease(code);
    }

    @Test
    void testPressKey_whenRobotIsAbsent_shouldNotInvokedKeyPress() {
        var keyCode = KeyCode.A;
        var robotService = new RobotService();

        robotService.pressKey(keyCode);

        verify(robot, times(0)).keyPress(isA(Integer.class));
        verify(robot, times(0)).keyRelease(isA(Integer.class));
    }
}
