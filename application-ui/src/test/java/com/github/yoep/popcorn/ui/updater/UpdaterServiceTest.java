package com.github.yoep.popcorn.ui.updater;

import com.fasterxml.jackson.module.paramnames.ParameterNamesModule;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformInfo;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformType;
import com.github.yoep.popcorn.backend.config.RestConfig;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.storage.StorageService;
import okhttp3.mockwebserver.MockResponse;
import okhttp3.mockwebserver.MockWebServer;
import org.apache.commons.io.IOUtils;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.boot.jackson.JsonComponentModule;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.task.TaskExecutor;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.Collections;
import java.util.concurrent.atomic.AtomicReference;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class UpdaterServiceTest {
    private static final MockWebServer MOCK_WEB_SERVER = new MockWebServer();

    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private PopcornProperties properties;
    @Mock
    private StorageService storageService;
    @Mock
    private TaskExecutor taskExecutor;

    private UpdateService updaterService;

    private final AtomicReference<Runnable> executorHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().when(properties.getUpdateChannel()).thenReturn(MOCK_WEB_SERVER.url("/").toString());
        lenient().doAnswer(invocation -> {
            executorHolder.set(invocation.getArgument(0, Runnable.class));
            return null;
        }).when(taskExecutor).execute(isA(Runnable.class));

        var restConfig = new RestConfig();
        var objectMapperBuilder = restConfig.jacksonObjectMapperBuilder(asList(
                new ParameterNamesModule(), new JsonComponentModule(), restConfig.javaTimeModule(), restConfig.jdk8Module()));

        updaterService = new UpdateService(platformProvider, properties, restConfig.webClient(objectMapperBuilder.createXmlMapper(false).build()), storageService, taskExecutor);
    }

    @Test
    void testInit_whenNewVersionIsAvailable_shouldSetStateUpdateAvailable() throws IOException {
        var newVersion = "9999.0.0";
        var currentVersion = "1.0.0";
        var versionInfo = VersionInfo.builder()
                .version(newVersion)
                .platforms(Collections.singletonMap("debian.x64", "update/9999.txt"))
                .changelog(Changelog.builder()
                        .features(new String[]{"my-new-feature"})
                        .bugfixes(new String[]{"my-bug-fix"})
                        .build())
                .build();
        MOCK_WEB_SERVER.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader(HttpHeaders.CONTENT_TYPE, MediaType.TEXT_PLAIN_VALUE)
                .setBody(readResourceFile("new-versions.json")));
        when(properties.getVersion()).thenReturn(currentVersion);
        when(platformProvider.platformInfo()).thenReturn(createPlatformInfo());

        updaterService.init();
        executorHolder.get().run();

        assertEquals(UpdateState.UPDATE_AVAILABLE, updaterService.getState());
        assertTrue(updaterService.getUpdateInfo().isPresent(), "Expected the update information to be available");
        assertEquals(versionInfo, updaterService.getUpdateInfo().get());
    }

    @Test
    void testInit_whenNoNewVersionIsAvailable_shouldCleanupExistingUpdateFile() throws IOException {
        var currentVersion = "1.0.0";
        var expectedName = UpdateService.DOWNLOAD_NAME + ".deb";
        MOCK_WEB_SERVER.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader(HttpHeaders.CONTENT_TYPE, MediaType.TEXT_PLAIN_VALUE)
                .setBody(readResourceFile("current-versions.json")));
        when(properties.getVersion()).thenReturn(currentVersion);
        when(platformProvider.platformInfo()).thenReturn(createPlatformInfo());

        updaterService.init();
        executorHolder.get().run();

        assertEquals(UpdateState.NO_UPDATE_AVAILABLE, updaterService.getState());
        verify(storageService).remove(expectedName);
    }

    @Test
    void testStartUpdateAndExit_whenInvoked_shouldExitPlatform() {

        updaterService.startUpdateAndExit();
        executorHolder.get().run();

        verify(platformProvider).exit();
    }

    private PlatformInfo createPlatformInfo() {
        return new PlatformInfo() {
            @Override
            public PlatformType getType() {
                return PlatformType.DEBIAN;
            }

            @Override
            public String getArch() {
                return "x64";
            }
        };
    }

    private static String readResourceFile(String name) throws IOException {
        var resource = new ClassPathResource(name);

        if (resource.exists()) {
            var versionsResource = resource.getInputStream();
            return IOUtils.toString(versionsResource, StandardCharsets.UTF_8);
        } else {
            throw new IOException("Resource " + name + " doesn't exist");
        }
    }
}