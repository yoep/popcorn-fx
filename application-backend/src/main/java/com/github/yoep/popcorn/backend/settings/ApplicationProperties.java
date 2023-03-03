package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.config.properties.ProviderProperties;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"updateChannel", "providers", "providersLen"})
public class ApplicationProperties extends Structure implements Closeable {
    public String updateChannel;
    public ProviderProperties.ByReference providers;
    public int providersLen;

    private List<ProviderProperties> providersCache;

    /**
     * Retrieve the provider properties based on the given name.
     *
     * @param name The name to retrieve the properties of.
     * @return Returns the provider properties.
     */
    public Optional<ProviderProperties> getProvider(String name) {
        return providersCache.stream()
                .filter(e -> e.name.equals(name))
                .findFirst();
    }

    @Override
    public void read() {
        super.read();
        providersCache = Optional.ofNullable(providers)
                .map(e -> (ProviderProperties[]) e.toArray(providersLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
        providersCache.forEach(ProviderProperties::close);
    }
}
