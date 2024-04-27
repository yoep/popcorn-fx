package com.github.yoep.popcorn;

import com.github.yoep.player.popcorn.controllers.components.*;
import com.github.yoep.player.popcorn.controllers.sections.PopcornPlayerSectionController;
import com.github.yoep.player.popcorn.player.EmbeddablePopcornPlayer;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.player.popcorn.services.*;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.lib.FxLibInstance;
import com.github.yoep.popcorn.backend.lib.PopcornFxInstance;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.ApplicationArgs;
import com.github.yoep.popcorn.ui.IoC;
import com.github.yoep.popcorn.ui.PopcornTimeApplication;
import com.github.yoep.popcorn.ui.PopcornTimePreloader;
import com.github.yoep.torrent.frostwire.*;
import com.github.yoep.video.javafx.VideoPlayerFX;
import com.github.yoep.video.vlc.VideoPlayerVlc;
import com.github.yoep.video.vlc.discovery.LinuxNativeDiscoveryStrategy;
import com.github.yoep.video.vlc.discovery.OsxNativeDiscoveryStrategy;
import com.github.yoep.video.vlc.discovery.WindowsNativeDiscoveryStrategy;
import com.github.yoep.video.youtube.VideoPlayerYoutube;
import com.sun.jna.StringArray;
import javafx.application.Application;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import javafx.application.Preloader;
import org.apache.http.impl.client.DefaultRedirectStrategy;
import org.apache.http.impl.client.HttpClientBuilder;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;
import uk.co.caprica.vlcj.factory.discovery.strategy.NativeDiscoveryStrategy;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.concurrent.atomic.AtomicBoolean;

public class PopcornTimeStarter {
    static final AtomicBoolean INIT_TORRENT_SERVICES = new AtomicBoolean(true);

    public static void main(String[] args) {
        System.setProperty("jna.encoding", StandardCharsets.UTF_8.name());
        var ioc = PopcornTimeApplication.getIOC();
        ioc.registerInstance(createApplicationArguments(args));

        var libArgs = createLibraryArguments(args);
        var fxLib = ioc.registerInstance(FxLibInstance.INSTANCE.get());
        var popcornFx = ioc.registerInstance(fxLib.new_popcorn_fx(libArgs.length, libArgs.args));
        FxLib.INSTANCE.set(fxLib);
        PopcornFxInstance.INSTANCE.set(popcornFx);

        PopcornTimeApplication.getON_INIT().set(PopcornTimeStarter::onInit);
        launch(PopcornTimeApplication.class, PopcornTimePreloader.class, args);
    }

    static void launch(Class<? extends Application> appClass, Class<? extends Preloader> preloaderClass, String... args) {
        System.setProperty("javafx.preloader", preloaderClass.getName());
        Application.launch(appClass, args);
    }

    static ApplicationArgs createApplicationArguments(String[] args) {
        return new ApplicationArgs(Arrays.stream(args)
                .filter(e -> !e.startsWith("--"))
                .toArray(String[]::new));
    }

    static ProgramArgs createLibraryArguments(String[] args) {
        var length = args.length + 1;
        var libArgs = new String[length];
        libArgs[0] = "popcorn-fx";
        System.arraycopy(args, 0, libArgs, 1, args.length);

        return new ProgramArgs(new StringArray(libArgs), length);
    }

    static void onInit(IoC ioC) {
        var applicationConfig = ioC.getInstance(ApplicationConfig.class);
        var discoveryStrategies = Arrays.<NativeDiscoveryStrategy>asList(
                new LinuxNativeDiscoveryStrategy(),
                new OsxNativeDiscoveryStrategy(),
                new WindowsNativeDiscoveryStrategy()
        );

        // torrent services
        if (INIT_TORRENT_SERVICES.get()) {
            initializeTorrentSession(ioC);
        }

        // youtube video player
        if (applicationConfig.isYoutubeVideoPlayerEnabled() && Platform.isSupported(ConditionalFeature.WEB)) {
            ioC.register(VideoPlayerYoutube.class);
        }

        // vlc video player
        if (applicationConfig.isVlcVideoPlayerEnabled()) {
            var discovery = new NativeDiscovery(discoveryStrategies.toArray(NativeDiscoveryStrategy[]::new));
            if (discovery.discover()) {
                ioC.registerInstance(discovery);
                ioC.register(MediaPlayerFactory.class);
                ioC.register(VideoPlayerVlc.class);
            }
        }

        // popcorn fx player
        ioC.register(EmbeddablePopcornPlayer.class);
        ioC.register(PlayerControlsService.class);
        ioC.register(PlayerHeaderComponent.class);
        ioC.register(PlayerHeaderService.class);
        ioC.register(PlayerPlaylistComponent.class);
        ioC.register(PlayerSubtitleComponent.class);
        ioC.register(PlayerSubtitleService.class);
        ioC.register(PopcornPlayer.class);
        ioC.register(PopcornPlayerSectionController.class);
        ioC.register(PopcornPlayerSectionService.class);
        ioC.register(SubtitleManagerService.class);
        ioC.register(VideoPlayerFX.class);
        ioC.register(VideoService.class);

        if (isDesktopMode(ioC)) {
            onInitDesktop(ioC);
        } else {
            onInitTv(ioC);
        }
    }

    private static void initializeTorrentSession(IoC ioC) {
        var client = HttpClientBuilder.create()
                .setRedirectStrategy(new DefaultRedirectStrategy())
                .build();

        ioC.register(TorrentServiceImpl.class);
        ioC.register(TorrentSessionManagerImpl.class);
        ioC.register(TorrentSettingsServiceImpl.class);
        ioC.registerInstance(new TorrentResolverService(ioC.getInstance(TorrentSessionManager.class), client));
    }

    private static void onInitDesktop(IoC ioC) {
        // popcorn fx player
        ioC.register(DesktopHeaderActionsComponent.class);
        ioC.register(DesktopPlayerControlsComponent.class);
    }

    private static void onInitTv(IoC ioC) {
        // popcorn fx player
        ioC.register(TvPlayerControlsComponent.class);
    }

    private static boolean isDesktopMode(IoC ioC) {
        return !ioC.getInstance(ApplicationConfig.class).isTvMode();
    }

    record ProgramArgs(StringArray args, int length) {
    }
}
