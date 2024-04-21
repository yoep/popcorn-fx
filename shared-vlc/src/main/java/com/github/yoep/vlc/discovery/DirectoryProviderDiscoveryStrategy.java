package com.github.yoep.vlc.discovery;

import lombok.extern.slf4j.Slf4j;
import uk.co.caprica.vlcj.factory.discovery.provider.DiscoveryDirectoryProvider;
import uk.co.caprica.vlcj.factory.discovery.strategy.BaseNativeDiscoveryStrategy;

import java.util.*;

import static java.util.Arrays.asList;

@Slf4j
abstract class DirectoryProviderDiscoveryStrategy extends BaseNativeDiscoveryStrategy {
    private static final String VLC_PATH_PROPERTY = "vlc.path";
    private static final String BUNDLED_LIB_PATH = System.getProperty("user.dir");

    /**
     * Service loader for the directory provider implementations.
     */
    private final ServiceLoader<DiscoveryDirectoryProvider> directoryProviders = ServiceLoader.load(DiscoveryDirectoryProvider.class);

    /**
     * Create a new native discovery strategy.
     *
     * @param filenamePatterns  filename patterns to search for, as regular expressions
     * @param pluginPathFormats directory name templates used to find the VLC plugin directory, printf style.
     */
    public DirectoryProviderDiscoveryStrategy(String[] filenamePatterns, String[] pluginPathFormats) {
        super(filenamePatterns, pluginPathFormats);
    }

    @Override
    public List<String> discoveryDirectories() {
        var vlcPathProperty = System.getProperty(VLC_PATH_PROPERTY);

        // check if a vlc path has been configured
        // if so, return the path as the exclusive search path for VLC
        if (vlcPathProperty != null && !vlcPathProperty.isBlank()) {
            log.debug("The following VLC path has been provided: {}", vlcPathProperty);
            return Collections.singletonList(vlcPathProperty);
        }

        var directories = new ArrayList<String>();

        // add the bundled library path as first path to search for
        directories.add(BUNDLED_LIB_PATH);

        for (DiscoveryDirectoryProvider provider : getSupportedProviders()) {
            directories.addAll(asList(provider.directories()));
        }

        log.trace("Directories which will be checked for VLC installation: {}", directories);
        return directories;
    }

    private List<DiscoveryDirectoryProvider> getSupportedProviders() {
        var result = new ArrayList<DiscoveryDirectoryProvider>();
        for (DiscoveryDirectoryProvider service : directoryProviders) {
            if (service.supported()) {
                result.add(service);
            }
        }
        return sort(result);
    }

    private List<DiscoveryDirectoryProvider> sort(List<DiscoveryDirectoryProvider> providers) {
        providers.sort(Comparator.comparingInt(DiscoveryDirectoryProvider::priority));
        return providers;
    }
}
