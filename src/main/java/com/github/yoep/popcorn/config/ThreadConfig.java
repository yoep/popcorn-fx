package com.github.yoep.popcorn.config;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;
import org.springframework.scheduling.annotation.EnableAsync;
import org.springframework.scheduling.concurrent.ThreadPoolTaskExecutor;

@Configuration
@EnableAsync
public class ThreadConfig {
    @Bean
    public TaskExecutor taskExecutor() {
        ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
        executor.setCorePoolSize(3);
        executor.setQueueCapacity(0);
        executor.setThreadNamePrefix("PT-background");
        executor.initialize();
        return executor;
    }

    @Bean
    public TaskExecutor uiTaskExecutor() {
        ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
        executor.setCorePoolSize(2);
        executor.setQueueCapacity(0);
        executor.setThreadNamePrefix("UI-background");
        executor.setThreadPriority(Thread.MAX_PRIORITY);
        executor.initialize();
        return executor;
    }
}
