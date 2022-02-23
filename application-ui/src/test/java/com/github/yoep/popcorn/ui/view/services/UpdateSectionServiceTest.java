package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.ui.updater.UpdateService;
import com.github.yoep.popcorn.ui.updater.UpdateState;
import com.github.yoep.popcorn.ui.updater.VersionInfo;
import com.github.yoep.popcorn.ui.view.listeners.UpdateListener;
import javafx.beans.property.SimpleObjectProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class UpdateSectionServiceTest {
    @Mock
    private UpdateService updateService;
    @Mock
    private UpdateListener listener;
    @InjectMocks
    private UpdateSectionService updateSectionService;

    private final SimpleObjectProperty<VersionInfo> updateInfo = new SimpleObjectProperty<>();
    private final SimpleObjectProperty<UpdateState> state = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        when(updateService.updateInfoProperty()).thenReturn(updateInfo);
        when(updateService.stateProperty()).thenReturn(state);
    }

    @Test
    void testUpdateInfoListener_whenUpdateInfoIsChanged_shouldInvokedListeners() {
        var versionInfo = VersionInfo.builder().build();
        updateSectionService.init();
        updateSectionService.addListener(listener);

        updateInfo.set(versionInfo);

        verify(listener).onUpdateInfoChanged(versionInfo);
    }

    @Test
    void testStateListener_whenStateIsChanged_shouldInvokedListeners() {
        var expectedState = UpdateState.DOWNLOADING;
        updateSectionService.init();
        updateSectionService.addListener(listener);

        state.set(expectedState);

        verify(listener).onUpdateStateChanged(expectedState);
    }
}