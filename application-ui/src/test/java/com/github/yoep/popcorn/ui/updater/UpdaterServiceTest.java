package com.github.yoep.popcorn.ui.updater;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.storage.StorageService;
import okhttp3.mockwebserver.MockWebServer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.web.reactive.function.client.WebClient;

@ExtendWith(MockitoExtension.class)
class UpdaterServiceTest {
    private static final MockWebServer MOCK_WEB_SERVER = new MockWebServer();
    private final WebClient webClient = WebClient.create(MOCK_WEB_SERVER.url("/").toString());

    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private PopcornProperties properties;
    @Mock
    private ObjectMapper objectMapper;
    @Mock
    private StorageService storageService;

    private UpdaterService updaterService;

    @BeforeEach
    void setUp() {
        updaterService = new UpdaterService(platformProvider, properties, webClient, objectMapper, storageService);
    }


}