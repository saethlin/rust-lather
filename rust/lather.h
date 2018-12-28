#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

/*
 * A model of a star with spots that can be observed.
 */
typedef struct Simulation Simulation;

/*
 * Add a spot to the simulation
 */
void simulation_add_spot(Simulation *sim,
                         double latitude,
                         double longitude,
                         double fill_factor,
                         bool plage);

/*
 * Remove all spots on this simulation
 */
void simulation_clear_spots(Simulation *sim);

/*
 * Close down a simulation
 */
void simulation_free(Simulation *sim);

/*
 * Build a simulation from a path to a config file
 */
Simulation *simulation_new(const char *filename, const char **error);

/*
 * Observe the flux of a simulation at given time values in days
 */
const double *simulation_observe_flux(Simulation *sim,
                                      double *times,
                                      uintptr_t n_times,
                                      double wave_start,
                                      double wave_end);

/*
 * Observe the rv and bisectors of a simulation at given time values in days
 */
const double *simulation_observe_rv(Simulation *sim,
                                    double *times,
                                    uintptr_t n_times,
                                    double wave_start,
                                    double wave_end);

/*
 * Print a simulation
 */
const char *simulation_tostring(Simulation *sim);
