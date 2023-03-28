package com.github.yoep.popcorn.ui.view;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationContext;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Consumer;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PopcornViewLoaderTest {
    @Mock
    private ApplicationContext applicationContext;
    @Mock
    private ViewManager viewManager;
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationConfig applicationConfig;

    @Test
    void testOnUiScaleChanged() {
        var newScale = 2.0f;
        var holder = new AtomicReference<Consumer<Float>>();
        doAnswer(invocation -> {
            holder.set(invocation.getArgument(0));
            return null;
        }).when(applicationConfig).setOnUiScaleChanged(isA(Consumer.class));
        var loader = new PopcornViewLoader(applicationContext, viewManager, localeText, applicationConfig);

        var consumer = holder.get();
        consumer.accept(newScale);

        assertEquals(newScale, loader.getScale());
    }
}