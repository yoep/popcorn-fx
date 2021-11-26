package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.controllers.components.PlayerHeaderComponent;
import com.github.yoep.player.popcorn.controllers.sections.PopcornPlayerSectionController;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

@ExtendWith(MockitoExtension.class)
class DetailsServiceTest {
    @Mock
    private VideoService videoService;
    @Mock
    private PopcornPlayerSectionController playerSectionController;
    @Mock
    private PlayerHeaderComponent playerHeaderComponent;
    @InjectMocks
    private DetailsService service;

    @Test
    void testInit_whenVideoPlayerIsSwitched_shouldUpdateVideoSurface() {
        service.init();
    }
}
