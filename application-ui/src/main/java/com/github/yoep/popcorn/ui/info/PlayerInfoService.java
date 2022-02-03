package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerInfoService {
    private final PlayerManagerService playerManagerService;
}
