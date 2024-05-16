package com.github.yoep.popcorn.ui;

import com.github.yoep.popcorn.backend.PopcornFx;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.lang.reflect.Constructor;
import java.lang.reflect.Modifier;
import java.lang.reflect.ParameterizedType;
import java.text.MessageFormat;
import java.util.*;
import java.util.concurrent.ExecutorService;
import java.util.stream.Collectors;

/**
 * A simple Inversion of Control (IoC) container implementation.
 * Allows registration of classes and instances, as well as retrieval of instances by type.
 */
@Slf4j
@ToString
public class IoC {
    private final Map<ComponentDefinition, Object> beans = new HashMap<>();
    private final Object lock = new Object();

    /**
     * Registers a class with the IoC container as a singleton.
     *
     * @param clazz the class to register
     */
    public void register(Class<?> clazz) {
        register(clazz, true);
    }

    /**
     * Registers a class with the IoC container.
     *
     * @param clazz       the class to register
     * @param isSingleton flag indicating whether the class should be treated as a singleton
     */
    public void register(Class<?> clazz, boolean isSingleton) {
        Objects.requireNonNull(clazz, "clazz cannot be null");
        doInternalRegister(clazz, null, isSingleton);
    }

    /**
     * Registers an instance with the IoC container as a singleton.
     *
     * @param instance the instance to register
     * @param <T>      the type of the instance
     * @return the registered instance
     */
    public <T> T registerInstance(T instance) {
        doInternalRegister(instance.getClass(), instance, true);
        return instance;
    }

    /**
     * Retrieves a singleton instance of the specified class from the IoC container.
     *
     * @param clazz the class of the instance to retrieve
     * @param <T>   the type of the instance
     * @return the singleton instance
     * @throws IoCException if no instance is found for the specified class
     */
    public <T> T getInstance(Class<T> clazz) {
        var result = doInternalGet(clazz);

        if (result.isEmpty()) {
            throw new IoCException(clazz, MessageFormat.format("No class bean found for {0}", clazz));
        }

        return result.get(0);
    }

    /**
     * Retrieves a singleton instance of the specified class from the IoC container.
     *
     * @param clazz the class of the instance to retrieve
     * @param <T>   the type of the instance
     * @return the singleton instance if found, else {@link Optional#empty()}
     */
    public <T> Optional<T> getOptionalInstance(Class<T> clazz) {
        var result = doInternalGet(clazz);
        return result.stream().findFirst();
    }

    /**
     * Retrieves all instances of the specified class from the IoC container.
     *
     * @param clazz the class of the instances to retrieve
     * @param <T>   the type of the instances
     * @return a list of instances
     */
    public <T> List<T> getInstances(Class<T> clazz) {
        return doInternalGet(clazz);
    }

    /**
     * Disposes resources managed by the IoC container.
     */
    public void dispose() {
        getInstance(PopcornFx.class).dispose();
        getInstance(ExecutorService.class).shutdownNow();

        beans.clear();
    }

    private void doInternalRegister(Class<?> clazz, Object instance, boolean isSingleton) {
        Objects.requireNonNull(clazz, "clazz cannot be null");
        if (clazz.isInterface()) {
            throw new IoCException(clazz, "Cannot register an interface type");
        }

        var componentDefinition = new ComponentDefinition(clazz, isSingleton);

        synchronized (lock) {
            if (!beans.containsKey(componentDefinition)) {
                log.trace("Registering new type {}", clazz);
                beans.put(componentDefinition, instance);
            } else {
                throw new IoCException(clazz, MessageFormat.format("Singleton type {0} has already been registered", clazz));
            }
        }
    }

    private <T> List<T> doInternalGet(Class<T> clazz) {
        synchronized (lock) {
            return beans.keySet().stream()
                    .filter(e -> e.defines(clazz))
                    .map(this::<T>doInternalGetInstance)
                    .collect(Collectors.toList());
        }
    }

    private <T> T doInternalGetInstance(ComponentDefinition definition) {
        var instance = this.beans.get(definition);

        if (instance == null || !definition.isSingleton()) {
            instance = initializeType(definition);
        }

        return (T) instance;
    }

    private Object initializeType(ComponentDefinition definition) {
        log.trace("Initializing type {}", definition);
        for (Constructor<?> constructor : definition.getType().getDeclaredConstructors()) {
            if (Modifier.isPublic(constructor.getModifiers())) {
                try {
                    var instance = constructor.newInstance(Arrays.stream(constructor.getGenericParameterTypes())
                            .map(e -> e instanceof ParameterizedType pt ? getInstances(getGenericType(pt)) : getInstance((Class<?>) e))
                            .toArray());
                    log.debug("Initialized new type instance {}", instance.getClass());

                    if (definition.isSingleton()) {
                        synchronized (lock) {
                            log.trace("Storing singleton type instance {}", definition);
                            beans.put(definition, instance);
                        }
                    }

                    return instance;
                } catch (Exception ex) {
                    throw new IoCException(definition.getType(), ex.getMessage(), ex);
                }
            }
        }

        throw new IoCException(definition.getType(), "Failed to initialize type instance");
    }

    private static Class<?> getGenericType(ParameterizedType parameterizedType) {
        var type = parameterizedType.getActualTypeArguments()[0];

        if (type instanceof ParameterizedType pt) {
            return (Class<?>) pt.getRawType();
        } else {
            try {
                return Class.forName(type.getTypeName());
            } catch (ClassNotFoundException ex) {
                throw new IoCException((Class<?>) parameterizedType.getRawType(), ex.getMessage(), ex);
            }
        }
    }

    /**
     * Represents the definition of a component registered in the IoC container.
     */
    @ToString
    @EqualsAndHashCode
    private static class ComponentDefinition {
        private final Class<?> type;
        private final List<Class<?>> derivedTypes;
        private final boolean isSingleton;

        /**
         * Constructs a new component definition.
         *
         * @param clazz       the class associated with the definition
         * @param isSingleton flag indicating whether the class should be treated as a singleton
         */
        public ComponentDefinition(Class<?> clazz, boolean isSingleton) {
            this.type = clazz;
            this.derivedTypes = discoverDerivedTypes(clazz);
            this.isSingleton = isSingleton;
        }

        public Class<?> getType() {
            return type;
        }

        public boolean isSingleton() {
            return isSingleton;
        }

        /**
         * Checks if the definition defines a class or its derived types.
         *
         * @param clazz the class to check
         * @return true if the class or its derived types are defined in the definition, otherwise false
         */
        public boolean defines(Class<?> clazz) {
            return derivedTypes.stream()
                    .anyMatch(clazz::isAssignableFrom);
        }

        private static List<Class<?>> discoverDerivedTypes(Class<?> clazz) {
            var types = new ArrayList<Class<?>>();

            types.add(clazz);
            for (Class<?> declaredClass : clazz.getDeclaredClasses()) {
                types.addAll(discoverDerivedTypes(declaredClass));
            }
            for (Class<?> declaredClass : clazz.getInterfaces()) {
                types.addAll(discoverDerivedTypes(declaredClass));
            }

            return types;
        }
    }
}
