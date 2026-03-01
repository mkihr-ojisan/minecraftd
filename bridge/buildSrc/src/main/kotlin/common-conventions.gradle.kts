plugins {
    id("base-conventions")
    id("com.gradleup.shadow")
}

val shade: Configuration by configurations.creating
configurations.compileClasspath.get().extendsFrom(shade)
configurations.runtimeClasspath.get().extendsFrom(shade)

dependencies {
    shade(project(":common:all"))
}

tasks.shadowJar {
    configurations = listOf(shade)
    exclude("META-INF")

    // PaperMC uses an older version of protobuf, so we need to relocate our protobuf dependency to avoid conflicts
    relocate("com.google.protobuf", "com.mkihr_ojisan.minecraftd_bridge.protobuf")
}

tasks.named("assemble") {
    dependsOn("shadowJar")
}