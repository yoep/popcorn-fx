package com.github.yoep.popcorn;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.IoC;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

@ExtendWith(MockitoExtension.class)
class PopcornTimeStarterTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx popcornFx;
    @Mock
    private ApplicationConfig applicationConfig;

    @Test
    void testOnInit() {
        var ioc = new IoC();
        ioc.registerInstance(fxLib);
        ioc.registerInstance(popcornFx);
        ioc.registerInstance(applicationConfig);

        PopcornTimeStarter.onInit(ioc);
    }
}